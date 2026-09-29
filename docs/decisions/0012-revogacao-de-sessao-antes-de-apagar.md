# 0012. Revogação de sessão antes de apagar

- Status: accepted
- Date: 2026-09-29
- Complementa: `0009-api-em-rust-com-axum-e-sqlx.md`, na sessão da `api-rs`.

## Context

A sessão mora em dois lugares que não fazem commit juntos. A linha em `sessions`, no Postgres, é o
que o refresh renova. O access token é um JWT de 15 minutos que a rota aceita sem ir ao banco, e só a
chave `auth:revoked:<sessão>` no Redis o recusa antes de vencer.

Três caminhos apagam a sessão: o logout, o reuso de um refresh token já rotacionado e o teto de 5
sessões por usuário no login. Em todos, o que não pode acontecer é a linha sumir com o access token
ainda aceito: o usuário saiu, ou o token vazou, e a API continua respondendo por até 15 minutos.

Até aqui os três abriam uma transação, apagavam, escreviam no Redis e só então faziam o commit. A
ordem estava certa, mas a transação ficava aberta, com a linha travada e a conexão presa, durante a
ida ao Redis, e no logout e no reuso ela não protegia nada que a ordem sozinha não protegesse.

## Decision

- **A revogação no Redis vem antes do `DELETE`, sempre.** Com o Redis fora, nada foi apagado e a rota
  responde `503`.
- **O logout e o reuso apagam em autocommit, sem transação**, porque sabem o id antes de apagar. O
  `close_session` revoga e depois apaga; o reuso desfaz a transação do `FOR UPDATE` e chama o
  `close_session`.
- **O teto de 5 mantém a transação.** Os ids a despejar só existem no `RETURNING` do `DELETE`, então a
  ordem é apagar, revogar e fazer o commit; com o Redis fora, o `DELETE` desfaz e o login responde
  `503`.
- **A chave vive o mesmo tempo que o access token** (`access::LIFETIME`), porque depois disso nenhum
  token daquela sessão passa pela assinatura.

## Consequences

Fica mais fácil:

- O logout e o reuso não seguram linha nem conexão durante a ida ao Redis.
- Toda falha parcial fecha a porta: se o `DELETE` falha depois da revogação, a linha fica, mas nenhum
  access token dela passa; o refresh até renova, e o token novo também é recusado até a chave vencer.

Fica mais difícil, e é aceito:

- **A ordem é garantida pelo código, sem teste.** Com o Redis fora, o `CurrentSession`, o limite da
  rota e o do refresh respondem `503` antes de chegar ao `DELETE`, então nenhuma requisição observa a
  ordem. Um teste exigiria um Redis que falha só na escrita.
- **Uma sessão revogada cujo `DELETE` falhou fica no banco até vencer.** Ela não serve para nada, e a
  purga de vencidas a remove.
- **O Redis precisa persistir.** Um Redis que reinicia vazio devolve a validade aos access tokens de
  sessões apagadas nos últimos 15 minutos.

## Alternatives considered

- **Manter a transação nos três caminhos** · rejeitada: trava a linha e prende a conexão durante uma ida
  à rede, sem proteger nada que a ordem não proteja.
- **Apagar primeiro e revogar depois, sem transação** · rejeitada: abre a janela em que a linha sumiu e o
  access token passa, e com o Redis fora ela dura os 15 minutos inteiros.
- **Consultar a sessão no Postgres em toda requisição autenticada** · rejeitada: troca o Redis por uma
  consulta por requisição, e o access token curto existe para evitá-la.
- **Outbox ou commit em duas fases entre Postgres e Redis** · rejeitada: o Redis não participa de
  transação distribuída, e a ordem resolve o que importa com uma escrita.

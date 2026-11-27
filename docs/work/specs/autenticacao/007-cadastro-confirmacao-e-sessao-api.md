# 007 — Criar a conta, confirmar o e-mail e manter a sessão (API)

> **Status:** closed
> **Perfil:** API
> **Módulo:** `apps/api`: `crates/core`, `crates/app` (features `users`, `email_verifications`,
> `sessions`, `credentials`, `email_outbox` e o encanamento `http/`) e o `crates/worker`, que nasce aqui
> **Épico:** #4 — Autenticação e sessão · issue #6
> **Requisitos:** FR-ACC-07, FR-ACC-08, FR-ACC-12, FR-ACC-15, NFR-SEC-07
> **Sprint:** 1
> **Decisões:** [ADR 0006](../../../decisions/0006-api-rest-e-problem-details.md) ·
> [ADR 0010](../../../decisions/0010-api-rs-com-crate-por-processo.md) ·
> [ADR 0011](../../../decisions/0011-openapi-da-api-rs-nasce-com-a-rota.md) ·
> [ADR 0012](../../../decisions/0012-revogacao-de-sessao-antes-de-apagar.md) ·
> [ADR 0017](../../../decisions/0017-fila-de-email-no-postgres-e-worker-com-cron.md)
> **Parte de:** [spec 003 · API](003-autenticacao-api.md), [spec 004 · API](004-sessao-por-token-api.md) e
> [spec 006](006-worker-e-fila-de-email-api.md), escritas para o código apagado. Esta spec as substitui
> no que cobre; o que ela não repete não vale.

## Premissas confirmadas

1. A sessão segue o ADR 0012: JWT de acesso de 15 minutos, refresh opaco rotacionado a cada uso (24 horas
   no web, 7 dias no app, nunca mais de 30 dias), denylist no Redis com falha fechada e no máximo 5
   sessões por usuário. Os cookies são `HttpOnly`, `SameSite=Lax`, `Secure` em produção e sem `Domain`.
2. O e-mail sai pela fila `email_outbox` do ADR 0017. O `crates/worker` nasce nesta entrega, com o envio
   por SMTP, as novas tentativas e a purga diária às 03:00 de São Paulo. O compose de dev ganha o
   Mailpit.
3. O limite principal é por alvo (falhas de login por e-mail, envios por endereço). O secundário é por
   IP, e o IP vem da fonte em `CLIENT_IP_SOURCE`. A guarda de `Origin` e o CORS com credenciais entram
   junto.
4. O transporte do app entra inteiro: `Clinicore-Client: mobile`, `Authorization: Bearer`, refresh no
   corpo, e cada token só vale pelo transporte em que a sessão nasceu.
5. As rotas são as da spec 003, reescritas em Rust. Nenhuma resposta revela quem tem conta, e toda
   resposta sai com `Cache-Control: no-store`. A `accounts` é separada da `users`, pronta para o Google
   da #7.
6. Google, recuperação e troca de senha e passkey ficam fora.
7. Uma branch e um PR, em quatro etapas de commit (a Quebra em tarefas).

## Critérios de aceite

### Contrato

| Método | Rota | Autenticação | Idempotente |
| --- | --- | --- | --- |
| `POST` | `/users` | pública | Sim: o e-mail é único, e repetir não grava nem envia (regra 8) |
| `POST` | `/email-verifications` | pública | Sim, dentro da janela de 60 segundos da regra 9 |
| `POST` | `/email-verifications/confirmation` | token no corpo | Sim: o token é consumido na primeira, e a segunda responde `400` sem gravar |
| `POST` | `/sessions` | pública | Não: cada login cria uma sessão, com o teto da regra 13 |
| `GET` | `/sessions/current` | sessão | Sim |
| `POST` | `/sessions/current/tokens` | refresh token | Não: rotaciona (regra 12) |
| `DELETE` | `/sessions/current` | sessão | Sim: depois da primeira, a sessão não existe e a repetição responde `401` sem alterar nada |

**Sessão** quer dizer o cookie `clinicore_access` válido sem `Clinicore-Client`, ou
`Authorization: Bearer <accessToken>` válido com `Clinicore-Client: mobile` (regra 11). `GET /health`
continua como o scaffold o deixou, sem limite e sem log.

### Requisição

Todo corpo é JSON. Campo desconhecido, ou de tipo errado, é recusado com `400 validation-failed` e o
`code` `invalid` no campo (ADR 0010). Nenhuma rota aceita URL de redirecionamento.

**Header `Clinicore-Client`**, lido em toda rota da tabela acima:

| Valor | Efeito |
| --- | --- |
| ausente | transporte do web, por cookie |
| `mobile` | transporte do app, por corpo e `Authorization` |
| qualquer outro | `400 invalid-client`, antes de qualquer outra conferência da rota |

**`POST /users`**

| Campo | Tipo | Obrigatório | Validação e origem |
| --- | --- | --- | --- |
| `name` | `string` | Sim | digitado pela pessoa; espaços das pontas removidos e normalizado em NFC; de 1 a 100 caracteres (`code` `length`); só letras, espaço, ponto, hífen e apóstrofo, com pelo menos uma letra (`code` `name`) |
| `email` | `string` | Sim | digitado pela pessoa; espaços das pontas removidos; endereço válido de até 254 caracteres (`code` `email`); gravado em minúsculas |
| `password` | `string` | Sim | digitada pela pessoa; a política da regra 5 (`code` `weak_password`) |

```json
{ "name": "Ana Souza", "email": "ana@exemplo.com", "password": "Clinica#2026" }
```

**`POST /email-verifications`**

| Campo | Tipo | Obrigatório | Validação e origem |
| --- | --- | --- | --- |
| `email` | `string` | Sim | digitado pela pessoa; as mesmas regras do `email` do cadastro |

**`POST /email-verifications/confirmation`**

| Campo | Tipo | Obrigatório | Validação e origem |
| --- | --- | --- | --- |
| `token` | `string` | Sim | o `token` da query do link do e-mail, repassado pelo cliente; exatamente 43 caracteres base64url sem preenchimento (`code` `invalid`) |

**`POST /sessions`**

| Campo | Tipo | Obrigatório | Validação e origem |
| --- | --- | --- | --- |
| `email` | `string` | Sim | digitado pela pessoa; as mesmas regras do `email` do cadastro |
| `password` | `string` | Sim | digitada pela pessoa; de 1 a 128 caracteres (`code` `length`), sem a política da regra 5 |

**`POST /sessions/current/tokens`**: sem `Clinicore-Client`, não aceita corpo e lê o cookie
`clinicore_refresh`. Com `Clinicore-Client: mobile`:

| Campo | Tipo | Obrigatório | Validação e origem |
| --- | --- | --- | --- |
| `refreshToken` | `string` | Sim | o `refreshToken` da última resposta de sessão, guardado pelo app; `<uuid>.<43 caracteres base64url>` (`code` `invalid`) |

**`GET /sessions/current`** e **`DELETE /sessions/current`**: sem corpo. Quem chama é identificado pela
sessão.

### Resposta

**`201 Created`, de `POST /sessions`**, com `Location: /sessions/current`. No web, junto vêm os dois
`Set-Cookie` da regra 10:

```json
{ "user": { "id": "0b8f2c1e-5a4d-4c3b-9e7f-1a2b3c4d5e6f", "name": "Ana Souza", "email": "ana@exemplo.com" } }
```

No app, nenhum `Set-Cookie`, e o corpo ganha os tokens:

```json
{
  "user": { "id": "0b8f2c1e-5a4d-4c3b-9e7f-1a2b3c4d5e6f", "name": "Ana Souza", "email": "ana@exemplo.com" },
  "tokens": {
    "accessToken": "<JWT>",
    "refreshToken": "<sessionId>.<segredo>",
    "accessTokenExpiresIn": 900
  }
}
```

**`200 OK`, de `GET /sessions/current`**: `{ "user": { … } }`, no formato acima, sem `Set-Cookie`.

**`200 OK`, de `POST /sessions/current/tokens` no app**: `{ "tokens": { … } }`, no formato acima. No web,
`204 No Content` com os dois `Set-Cookie` novos.

**`202 Accepted` com corpo vazio**, de `POST /users` e `POST /email-verifications`, sempre (regra 8).

**`204 No Content`**, de `POST /email-verifications/confirmation`, sem sessão e sem cookie, e de
`DELETE /sessions/current`, com os dois `Set-Cookie` de expiração no web e nenhum no app.

Os schemas do OpenAPI são `SignUpRequest`, `EmailVerificationRequest`,
`EmailVerificationConfirmationRequest`, `SignInRequest`, `RefreshTokensRequest`, `SessionResponse`,
`RefreshTokensResponse` (o `{ "tokens": … }` do refresh do app), `UserResponse` e `TokensResponse`.

**Erro**, em toda rota: Problem Details da RFC 9457, em `application/problem+json`, com
`type` `tag:clinicore.com.br,2026:<slug>`, o `title` fixo da seção Erros e o `status`. A validação traz
`errors`, um item por campo:

```json
{
  "type": "tag:clinicore.com.br,2026:validation-failed",
  "title": "Validation failed",
  "status": 400,
  "errors": [{ "pointer": "#/password", "code": "weak_password" }]
}
```

| Status | Quando |
| --- | --- |
| `200` | Leitura da sessão; refresh do app |
| `201` | Login |
| `202` | Cadastro e pedido de link |
| `204` | Confirmação, refresh do web e logout |
| `400` | Corpo inválido, `Clinicore-Client` desconhecido, token do link inválido ou vencido |
| `401` | Credenciais incorretas; sessão ausente, inválida, vencida ou revogada; refresh reusado |
| `403` | E-mail não confirmado; `Origin` recusada |
| `429` | Limite excedido |
| `500` | Falha interna, sem detalhe |
| `503` | Redis fora |

### Perfis e privilégios

| Papel | Permissão | Observação |
| --- | --- | --- |
| Visitante sem sessão | nenhuma | cria a conta, pede o link, confirma o e-mail e entra |
| Pessoa com sessão | sessão válida | lê a própria sessão, renova e sai |

Não existe papel nesta entrega: a pessoa com sessão não pertence a nenhuma clínica. Papéis, alcance e
tenancy (`FR-ACC-01` a `05`, `NFR-SEC-01`) entram em outro épico.

---

## Regras de negócio

### 1. O cliente se declara, e cada transporte só vale por si

- O cliente se declara pelo header `Clinicore-Client` (tabela da Requisição). A API nunca deduz o cliente
  por `User-Agent` nem pela falta de `Origin`.
- O valor declarado escolhe o transporte da sessão, e a sessão grava em `sessions.client` o transporte em
  que nasceu (`web` ou `mobile`).
- O access token carrega o claim `cli` com esse valor. A API recusa com `401 invalid-session` o token
  cujo `cli` difere do transporte da requisição, e o refresh token cuja sessão nasceu no outro
  transporte, sem apagar a sessão.

### 2. A origem é conferida, e o CORS só libera a lista

- `ALLOWED_ORIGINS` é a lista de origens exatas que falam com a API pelo navegador. O CORS responde só
  a essas origens, com `Access-Control-Allow-Credentials: true`, os métodos `GET`, `POST`, `PUT`,
  `PATCH` e `DELETE`, os headers `Content-Type`, `Authorization` e `Clinicore-Client`, e
  `Access-Control-Max-Age: 600`. Nunca `*`.
- **A guarda de origem** roda em todo método que não é `GET`, `HEAD` nem `OPTIONS`, e deixa passar
  quando:
  1. o header `Origin` é igual a um item de `ALLOWED_ORIGINS`, comparado por igualdade da string
     inteira, nunca por sufixo; ou
  2. não há `Origin` nem `Cookie`: é o app, que não tem credencial que o navegador anexe sozinho.
- O resto responde `403 invalid-origin` antes do limite e antes do handler, sem gastar contagem.
- `APP_ORIGIN` é a origem do web e a única de onde a API monta um link: o do e-mail. Ela é uma das
  origens de `ALLOWED_ORIGINS`, e o boot recusa uma `APP_ORIGIN` que não esteja na lista.

### 3. Nenhuma resposta da API fica no cache

- Toda resposta, erro e `204` inclusive, sai com `Cache-Control: no-store` (NFR-SEC-07). A camada fica
  no `Router` e cobre também o `404` de rota inexistente e o `GET /health`.

### 4. O IP do cliente vem de uma fonte declarada

- `CLIENT_IP_SOURCE` diz de onde a API lê o IP:

  | Valor | IP lido | Uso |
  | --- | --- | --- |
  | `peer` | o endereço da conexão TCP | desenvolvimento e teste, sem proxy |
  | `cf-connecting-ip` | o header `CF-Connecting-IP` | produção atrás do Cloudflare Tunnel |
  | `x-forwarded-for:<n>` | a n-ésima entrada a partir da direita do `X-Forwarded-For`, com `n` de 1 a 5 | atrás de `n` proxies reversos que acrescentam o header |

- O que o cliente escreve à esquerda das entradas acrescentadas pelos proxies nunca é lido.
- Header ausente, com menos entradas que `n`, ou com um valor que não é IP: a requisição usa o
  contador fixo `unknown` e divide o limite com as outras na mesma condição, em vez de escapar dele. O log
  registra `warn` `client ip unavailable`, sem o valor do header.
- O IPv4 mapeado em IPv6 (`::ffff:`) é lido como IPv4. Um IPv6 conta pelo `/64` dele.
- A garantia depende do deploy: com `cf-connecting-ip`, a porta da API não é publicada no host. Isso é
  do deploy (Fora de escopo).

### 5. A política de senha

- A senha é aceita quando tem de 8 a 128 caracteres, ao menos uma letra maiúscula, ao menos um dígito e
  ao menos um caractere que não seja letra nem dígito. Senão, `400 validation-failed` com o `code`
  `weak_password` em `#/password`.
- A regra é uma função pura em `credentials/`, usada pelo validador do cadastro; a #7 a reaproveita na
  redefinição e na troca.
- A senha vira hash Argon2id com os parâmetros padrão do crate `argon2`, guardado em
  `accounts.password_hash` e nunca em `users`.

### 6. O cadastro cria a conta sem sessão e pede a confirmação

- `POST /users` com um e-mail que não tem conta grava, numa transação: o usuário com
  `email_verified = false`, a conta `credential` com o hash da senha, a reserva de envio da regra 9, a
  verificação da regra 7 e a linha da `email_outbox` com o e-mail da regra 15. Responde `202` sem corpo
  e sem cookie (FR-ACC-07.1).
- O cadastro nunca abre sessão.

### 7. O link de confirmação aponta para o web e vale 1 hora

- O segredo do link são 32 bytes aleatórios em base64url sem preenchimento, 43 caracteres. A
  `verifications` guarda só o SHA-256 dele, com `purpose = 'email_verification'` e `expires_at` 1 hora
  depois da emissão.
- O link é `<APP_ORIGIN>/verify-email?token=<segredo>`. A página do web chama
  `POST /email-verifications/confirmation` com o token no corpo. O link nunca aponta para a API: um `GET`
  que consome o token seria gasto pelo leitor de links do cliente de e-mail.
- A confirmação procura a verificação pelo hash do token, sob lock pelo endereço:
  - válida: marca `users.email_verified = true`, consome todas as verificações pendentes do endereço com
    o mesmo `purpose` e responde `204`, sem sessão e sem cookie (FR-ACC-15.1);
  - consumida ou inexistente: `400 invalid-token`, sem gravar nada;
  - vencida: `400 token-expired`, sem gravar nada.
- A confirmação não abre sessão porque quem cadastrou o endereço de outra pessoa escolheu a senha. A
  pessoa entra depois, com a senha.

### 8. Nenhuma resposta revela quem tem conta

- `POST /users` e `POST /email-verifications` respondem `202` sem corpo para e-mail novo, já cadastrado,
  já confirmado, sem conta, dentro ou fora do limite da regra 9.
- Cadastro de um e-mail que já tem conta não grava nada e não envia nada. Dois cadastros simultâneos do
  mesmo e-mail novo: o segundo cai no `ON CONFLICT (email) DO NOTHING`, desfaz a transação e responde
  `202` igual.
- `POST /email-verifications` só grava verificação e e-mail quando o endereço é de uma conta ainda não
  confirmada, mas registra a reserva de envio da regra 9 para qualquer endereço.
- `POST /sessions` responde `401 invalid-credentials` igual para e-mail sem conta e para senha errada
  (FR-ACC-08.2). Sem conta, a API confere a senha contra um hash Argon2id fixo, gerado no boot, e
  descarta o resultado, para o tempo de resposta não diferir.
- Nenhum e-mail avisa o dono do endereço de uma tentativa de cadastro.

### 9. Cada endereço recebe no máximo 1 e-mail por minuto e 5 por dia

- Por endereço, em qualquer IP, por tipo: no máximo 1 envio a cada 60 segundos e 5 a cada 24 horas,
  contados na `email_dispatches` com `kind = 'email_verification'`.
- A contagem e a gravação acontecem na mesma transação, sob `pg_advisory_xact_lock` pelo endereço, então
  dois pedidos simultâneos não passam os dois.
- Os três caminhos que pedem o e-mail de confirmação reservam antes de gravar o token: o cadastro, o
  pedido de link e o login com e-mail não confirmado.
- O pedido barrado não grava token nem e-mail, e a rota responde o mesmo que responderia enviando:
  `202`, ou `403 email-not-verified` no login.

### 10. A sessão do web são dois cookies

| Cookie | Conteúdo | `Path` | `Max-Age` |
| --- | --- | --- | --- |
| `clinicore_access` | o JWT de acesso | `/` | 900 |
| `clinicore_refresh` | `<sessionId>.<segredo>` | `/sessions/current/tokens` | 86400 |

- Os dois são `HttpOnly` e `SameSite=Lax` em todo ambiente, `Secure` quando `APP_ENV = production`, e
  não têm `Domain`. O JavaScript do web nunca os lê.
- O `Path` do refresh faz o cookie acompanhar só a rota de refresh. O logout não precisa dele: o id da
  sessão está no JWT.
- O logout responde os dois cookies com `Max-Age=0`, nos mesmos `Path`.

### 11. O app recebe a sessão no corpo

- Com `Clinicore-Client: mobile`, o login e o refresh devolvem `tokens` no corpo (Resposta), e a API
  nunca escreve cookie.
- O app manda `Authorization: Bearer <accessToken>`. Com `Clinicore-Client: mobile` a API lê o token só
  desse header; sem o header, só do cookie.
- `accessTokenExpiresIn` é sempre `900`.

### 12. O access token é curto, e o refresh rotaciona

- O access token é um JWT HS256 assinado com `JWT_SECRET`, com os claims `sub` (o id do usuário), `sid`
  (o id da sessão), `cli` (`web` ou `mobile`), `iat` e `exp`, 900 segundos depois do `iat`. Nada mais:
  nome e e-mail saem de `GET /sessions/current`.
- A requisição autenticada confere a assinatura, o `exp` e o `cli`, e consulta `auth:revoked:<sid>` no
  Redis, sem ir ao PostgreSQL. Token ausente, malformado, vencido, revogado ou de outro transporte
  responde `401 invalid-session`.
- O refresh token é `<sessionId>.<segredo>`, com 32 bytes aleatórios em base64url. A `sessions` guarda só
  o SHA-256 do segredo, comparado em tempo constante.
- `POST /sessions/current/tokens` consulta a denylist e, com a linha da sessão sob `FOR UPDATE`:
  1. sessão inexistente, vencida, com mais de 30 dias desde o `created_at`, revogada ou de outro
     transporte: `401 invalid-session`, sem gravar nada;
  2. hash igual: grava um segredo novo, empurra o `expires_at` 24 horas (web) ou 7 dias (app) a partir de
     agora, sem passar dos 30 dias do `created_at`, e devolve os tokens novos;
  3. hash diferente: é reuso. A API desfaz a transação, revoga e apaga a sessão na ordem do ADR 0012 e
     responde `401 session-reused`.
- Duas renovações simultâneas com o mesmo refresh válido: uma rotaciona e a outra cai no reuso e derruba
  a sessão. O custo é um novo login.
- **A revogação segue o ADR 0012:** grava `auth:revoked:<sessionId>` com 7 dias de vida antes de apagar a
  linha. Com o Redis fora, nada é apagado e a rota responde `503 service-unavailable`.

### 13. O login abre a sessão só para a conta confirmada, e no máximo 5 por usuário

- `POST /sessions`, nesta ordem: conta a tentativa por e-mail (regra 14), confere a senha (regra 8) e:
  - senha errada ou e-mail sem conta: `401 invalid-credentials`;
  - senha certa e e-mail não confirmado: reserva o reenvio (regra 9) e, se passar, grava verificação e
    e-mail novos; responde `403 email-not-verified` sem sessão nos dois casos (FR-ACC-15.2);
  - senha certa e e-mail confirmado: abre a sessão e responde `201` (FR-ACC-08.1).
- A sessão nasce com `client`, o hash do refresh, `expires_at` de 24 horas (web) ou 7 dias (app),
  `ip_address` (regra 4, nulo quando o IP é `unknown`) e `user_agent` cortado em 512 caracteres.
- Na mesma transação, com o usuário sob `FOR UPDATE`, a API apaga as sessões do usuário além das 5 mais
  recentes, nunca a recém-aberta, e revoga cada id apagado antes do commit (ADR 0012). O teto conta web e
  app juntos. Com o Redis fora, a transação desfaz e o login responde `503 service-unavailable`.

### 14. Os limites contam em duas chaves, e a falha é fechada

- Toda rota da tabela do Contrato é limitada no Redis, por janela fixa:

  | Rota | Chave | Janela | Máximo |
  | --- | --- | --- | --- |
  | `POST /users` | IP | 60 s | 3 |
  | `POST /users` | endereço | regra 9 | regra 9 |
  | `POST /email-verifications` | IP | 60 s | 3 |
  | `POST /email-verifications` | endereço | regra 9 | regra 9 |
  | `POST /email-verifications/confirmation` | IP | 60 s | 10 |
  | `POST /email-verifications/confirmation` | faixa de rede do IP: `/24` no IPv4, `/48` no IPv6 | 60 s | 300 |
  | `POST /sessions` | IP | 60 s | 5 |
  | `POST /sessions` | falhas de senha pelo e-mail, em minúsculas | 15 min | 10 |
  | `POST /sessions/current/tokens` | IP | 60 s | 30 |
  | `POST /sessions/current/tokens` | a sessão do refresh token | 60 s | 30 |
  | `GET` e `DELETE /sessions/current` | IP | 10 s | 100 |
  | `GET` e `DELETE /sessions/current` | a sessão do access token | 10 s | 100 |

- **O contador de falhas do login conta antes do Argon2**, e devolve o ponto quando a senha confere.
  Com 10 falhas na janela, o login daquele e-mail responde `429 rate-limited` mesmo com a senha certa,
  até a janela vencer. Travar o e-mail de outra pessoa por 15 minutos é o custo aceito.
- Excedido qualquer limite, a rota responde `429 rate-limited` com `Retry-After` em segundos até o fim da
  janela.
- A ordem é: cliente, origem, limite por IP, corpo, limite por alvo, handler. A rota autenticada conta a
  sessão depois de conferir o token.
- Redis inalcançável durante a contagem responde `503 service-unavailable`. O `GET /health` não conta.

### 15. O e-mail de confirmação

- A API renderiza o e-mail e grava assunto, HTML e texto na `email_outbox`, na transação da verificação,
  com o mesmo `expires_at`. Se a transação desfaz, não existe token nem e-mail.
- Remetente: `MAIL_FROM`. Assunto: "Confirme seu e-mail no Clinicore".
- Corpo, em pt-BR, com o primeiro nome cortado em 30 caracteres:

  > "Olá, {primeiro nome}. Confirme seu e-mail para começar a usar o Clinicore. O link expira em 1 hora."

  com o botão "Confirmar e-mail" apontando para o link da regra 7. A versão em texto traz a mesma frase e
  o link numa linha própria. Os templates são `askama`, no `crates/core`.

### 16. O worker envia o que está na fila

- O `crates/worker` (pacote `clinicore-worker`, binário `worker`) depende só do `clinicore-core` e é o
  único processo que fala com o SMTP.
- A cada 5 segundos, reserva até 10 linhas `pending` com `next_attempt_at` até agora e `expires_at`
  depois de agora, em ordem de `next_attempt_at`, com `FOR UPDATE SKIP LOCKED`. A reserva soma 1 a
  `attempts`, empurra `next_attempt_at` 2 minutos e é comitada antes do envio.
- Envia uma linha por vez pelo `lettre`, com pool e 10 segundos de timeout. A linha enviada é apagada.
- A falha volta com espera de 1, 2, 4 e 8 minutos, e registra `warn` `mail delivery failed, retrying`
  com `email` mascarado (`a***@exemplo.com`), `attempt` e `reason`. A quinta falha marca `failed`, apaga
  os dois corpos na mesma escrita e registra `error` `mail delivery gave up` com `email` mascarado,
  `attempts` e `reason`.
- A linha vencida não é enviada: espera a purga.
- A consulta de reserva que falha registra `error` `could not claim the pending mail`, com `error`, e o
  worker tenta de novo em 5 segundos.
- Com `SIGTERM` ou `Ctrl+C`, o worker para de reservar, termina o envio em curso, grava o resultado e
  sai com código 0.
- O SMTP fora do ar não muda a resposta da API, que já respondeu.

### 17. A purga roda uma vez por dia, às 03:00 de São Paulo

- O worker calcula a próxima `0 3 * * *` em `America/Sao_Paulo` pelo `croner` sobre o `jiff`, com o
  banco de fusos embutido, dorme até ela, purga e repete. Ao subir, não purga.
- Apaga:

  | Tabela | Condição |
  | --- | --- |
  | `sessions` | `expires_at` até agora, ou `created_at` até agora menos 30 dias |
  | `verifications` | `expires_at` até agora |
  | `email_dispatches` | `created_at` até agora menos 24 horas |
  | `email_outbox` | `expires_at` até agora |

- A sessão purgada não vai para a denylist: o access token dela já venceu.
- Cada execução registra `info` `purged the expired authentication records`, com os campos `sessions`,
  `verifications`, `email_dispatches` e `email_outbox`. A falha registra `error`
  `could not purge the expired authentication records`, com `error`, e espera a 03:00 seguinte.
- Duas instâncias purgam duas vezes, e a segunda apaga zero linhas.
- Os prazos `SESSION_ABSOLUTE_LIFETIME` (30 dias) e `DISPATCH_WINDOW` (24 horas) moram em
  `crates/core/src/lifetimes.rs`, usados pela API e pelo worker.

### 18. Toda configuração é obrigatória e validada no boot

- A API e o worker leem o ambiente só em `crates/core/src/config.rs`, sem valor padrão. Faltando ou
  inválida qualquer variável, o processo escreve `Invalid environment:` e uma linha por variável
  recusada, em ordem alfabética, e sai com código 1, antes de abrir conexão. O valor recebido nunca é
  impresso.
- Variáveis novas da API:

  | Nome | Formato esperado |
  | --- | --- |
  | `ALLOWED_ORIGINS` | `expected a comma-separated list of origins (https://host[:port])` |
  | `APP_ORIGIN` | `expected an origin listed in ALLOWED_ORIGINS` |
  | `CLIENT_IP_SOURCE` | `expected one of peer, cf-connecting-ip, x-forwarded-for:<1-5>` |
  | `JWT_SECRET` | `expected a string with at least 32 characters` |
  | `REDIS_URL` | `expected a Redis connection string (redis://…)` |

- O worker lê `APP_ENV`, `LOG_LEVEL`, `DATABASE_URL` e mais:

  | Nome | Formato esperado |
  | --- | --- |
  | `MAIL_FROM` | `expected a mailbox (Name <address@domain>)` |
  | `MAIL_REDIRECT_TO` | `expected off, or an address outside production` |
  | `SMTP_HOST` | `expected a hostname` |
  | `SMTP_PASSWORD` | `expected a non-empty string` |
  | `SMTP_PORT` | `expected an integer between 1 and 65535` |
  | `SMTP_TLS` | `expected one of starttls, tls, none; none only outside production` |
  | `SMTP_USER` | `expected a non-empty string` |

- A API não lê nenhuma `SMTP_*` nem `MAIL_FROM`.
- O compose de dev ganha o Mailpit, com `MP_SMTP_AUTH_ACCEPT_ANY=1` e
  `MP_SMTP_AUTH_ALLOW_INSECURE=1`, nas portas `MAILPIT_SMTP_PORT` e `MAILPIT_UI_PORT`, ligadas só a
  `127.0.0.1`, para quem quer testar sem SMTP de verdade.
- Em dev, o worker envia pelo SMTP do ambiente, e o `MAIL_REDIRECT_TO` troca o destinatário de todo
  e-mail por um endereço só, guardado no Infisical; nenhum e-mail chega ao endereço da conta. Fora de
  `production` ele aceita `off` ou um endereço; em `production`, só `off`.
- O `.env.example` lista as variáveis novas, e o CI da API recebe o Redis como serviço.

### 19. Persistência, auditoria e log

- **Tabelas novas**, numa migration por etapa da Quebra em tarefas: `users`, `accounts`, `sessions`,
  `verifications`, `email_dispatches` e `email_outbox`, com as colunas do Impacto no modelo de dados.
- **Auditoria:** N/A. Esta entrega não toca dado clínico, e a trilha de auditoria é o `FR-ACC-06`, de
  outro épico.
- **Log:** o log de requisição continua registrando só método, caminho, status e duração. Nenhum header,
  corpo, cookie ou token entra em log, e o `config` nunca é passado ao `tracing`.
- **Integrações:** o SMTP, só pelo worker.

---

## Erros

O `title` é inglês, texto de desenvolvedor. O texto que a pessoa lê é escrito em cada cliente, a partir
do slug e do `code`.

| Slug | HTTP | Quando | `title` |
| --- | --- | --- | --- |
| `validation-failed` | `400` | corpo fora do contrato; `errors` traz `pointer` e `code` por campo | "Validation failed" |
| `invalid-client` | `400` | `Clinicore-Client` com valor diferente de `mobile` | "Invalid client" |
| `invalid-token` | `400` | token do link inexistente ou já consumido | "Invalid token" |
| `token-expired` | `400` | token do link vencido | "Token expired" |
| `invalid-credentials` | `401` | e-mail sem conta, ou senha errada | "Invalid email or password" |
| `invalid-session` | `401` | access token ausente, malformado, vencido, revogado ou de outro transporte; refresh de sessão inexistente, vencida, revogada ou de outro transporte | "Invalid session" |
| `session-reused` | `401` | refresh cujo segredo não é o atual; a sessão é derrubada | "Refresh token reuse detected" |
| `email-not-verified` | `403` | login com a senha certa e o e-mail não confirmado | "Email not verified" |
| `invalid-origin` | `403` | método que muda estado com `Origin` fora da lista, ou sem `Origin` e com `Cookie` | "Invalid origin" |
| `rate-limited` | `429` | limite da regra 14 excedido | "Too many requests" |
| `service-unavailable` | `503` | Redis inalcançável | "Service temporarily unavailable" |

Todo `401` traz `WWW-Authenticate: Bearer realm="clinicore"`. Corpo JSON malformado, rota inexistente,
método não aceito e corpo grande demais saem como `about:blank`, com a frase padrão do status. O `500`
sai `about:blank` sem detalhe, e a causa vai só para o log.

## Efeitos colaterais

- **Persistência:**
  - o cadastro de e-mail novo grava uma linha em `users`, `accounts`, `email_dispatches`,
    `verifications` e `email_outbox`; o de e-mail existente não grava nada;
  - o pedido de link aceito grava em `email_dispatches` e, se a conta existe e não está confirmada, em
    `verifications` e `email_outbox`;
  - a confirmação marca `users.email_verified` e grava `consumed_at` nas verificações pendentes do
    endereço;
  - o login grava em `sessions` e apaga as excedentes; o refresh atualiza `refresh_token_hash` e
    `expires_at`; o reuso e o logout apagam a linha;
  - o worker apaga a linha enviada, adia ou marca `failed` a que falhou, e a purga apaga o que venceu.
- **Redis:** um contador com a vida da janela por chave limitada, e `auth:revoked:<sessionId>` com 7
  dias por sessão revogada.
- **Concorrência:**
  - dois cadastros do mesmo e-mail novo: o segundo cai no `ON CONFLICT` e responde `202` (regra 8);
  - dois pedidos de e-mail para o mesmo endereço: o lock por endereço serializa, e o segundo é barrado
    dentro da janela (regra 9);
  - duas confirmações do mesmo token: o lock por endereço serializa, e a segunda responde
    `400 invalid-token`;
  - dois refreshes do mesmo token: o `FOR UPDATE` serializa, e o segundo derruba a sessão (regra 12);
  - dois logins do mesmo usuário: o `FOR UPDATE` no usuário serializa o corte, e ficam as 5 mais recentes
    (regra 13);
  - dois workers: o `SKIP LOCKED` impede reservar a mesma linha (regra 16).
- **Transação:** cadastro (`users`, `accounts`, `email_dispatches`, `verifications`, `email_outbox`);
  pedido de link e reenvio do login (`email_dispatches`, `verifications`, `email_outbox`); confirmação;
  abertura de sessão com o corte do teto; rotação do refresh; reserva do worker, comitada antes do SMTP;
  e o resultado do envio. O SMTP nunca está dentro de uma transação.

---

## Cenários de aceite (Gherkin)

Os testes da API sobem o `Router` inteiro e chamam as rotas pelo `tower::ServiceExt`, contra o
PostgreSQL e o Redis reais do compose, com um banco novo por teste a partir das migrations. Os do worker
rodam o job contra o mesmo PostgreSQL e um SMTP de teste local que grava as mensagens recebidas. Nada
além do SMTP é substituído.

### Cenário 1 — O cadastro cria a conta sem sessão e enfileira o link (caminho feliz, regras 6, 7 e 15)

```gherkin
Dado que não existe conta para "ana@exemplo.com"
Quando o cliente envia `POST /users` com nome "Ana Souza", e-mail "Ana@Exemplo.com" e senha "Clinica#2026"
Então a API responde `202` sem corpo e sem `Set-Cookie`
E existe um usuário "ana@exemplo.com" com `email_verified` falso e uma conta `credential` com hash Argon2id
E existe uma verificação `email_verification` que vence em 1 hora, guardada só como hash
E a `email_outbox` tem uma linha `pending` para "ana@exemplo.com" com o assunto "Confirme seu e-mail no Clinicore"
E o texto do e-mail traz "Olá, Ana." e o link `<APP_ORIGIN>/verify-email?token=` seguido de 43 caracteres
```

### Cenário 2 — Cadastro repetido não grava, não envia e não revela a conta (exceção, regra 8)

```gherkin
Dado uma conta para "ana@exemplo.com"
Quando o cliente envia `POST /users` com o mesmo e-mail e outra senha válida
Então a API responde `202` sem corpo
E nenhuma linha nova existe em `users`, `accounts`, `verifications`, `email_dispatches` ou `email_outbox`
```

### Cenário 3 — Dois cadastros simultâneos do mesmo e-mail criam uma conta só (exceção, regra 8)

```gherkin
Dado que não existe conta para "ana@exemplo.com"
Quando dois `POST /users` com esse e-mail chegam ao mesmo tempo
Então as duas respostas são `202` sem corpo
E existe um usuário, uma conta e uma linha na `email_outbox` para o endereço
```

### Cenário 4 — Corpo inválido é recusado campo a campo (exceção, regras 5 e 6)

```gherkin
Quando o cliente envia `POST /users` com nome "", e-mail "ana" e senha "fraca"
Então a API responde `400` com `type` `validation-failed`
E `errors` traz `#/name` com `code` `length`, `#/email` com `email` e `#/password` com `weak_password`
E nada é gravado
E um corpo com o campo extra "role" responde `400` com `#/role` e `code` `invalid`
```

### Cenário 5 — Confirmar o e-mail libera o login sem abrir sessão (caminho feliz, regra 7)

```gherkin
Dado uma conta não confirmada com duas verificações pendentes, e o segredo da mais recente
Quando o cliente envia `POST /email-verifications/confirmation` com esse segredo
Então a API responde `204` sem `Set-Cookie`
E o usuário tem `email_verified` verdadeiro
E as duas verificações têm `consumed_at`
E repetir a confirmação com o mesmo segredo responde `400` `invalid-token`
```

### Cenário 6 — Token desconhecido e token vencido são recusados (exceção, regra 7)

```gherkin
Dado uma verificação emitida há 61 minutos
Quando o cliente confirma com o segredo dela
Então a API responde `400` `token-expired` e o usuário continua não confirmado
E confirmar com 43 caracteres que nenhuma verificação tem responde `400` `invalid-token`
E confirmar com 42 caracteres responde `400` `validation-failed` com `#/token` e `code` `invalid`
```

### Cenário 7 — Pedir um novo link responde igual em todo caso (caminho alternativo, regras 8 e 9)

```gherkin
Dado uma conta não confirmada "ana@exemplo.com" e uma confirmada "bia@exemplo.com"
Quando o cliente envia `POST /email-verifications` para "ana@exemplo.com", "bia@exemplo.com" e "sem@exemplo.com"
Então as três respostas são `202` sem corpo
E só "ana@exemplo.com" ganha verificação nova e linha na `email_outbox`
E os três endereços ganham uma linha em `email_dispatches`
```

### Cenário 8 — O limite por endereço barra o segundo pedido no minuto e o sexto no dia (exceção, regra 9)

```gherkin
Dado uma conta não confirmada "ana@exemplo.com" com um envio registrado há 30 segundos
Quando o cliente pede um novo link para ela
Então a API responde `202` e nada é gravado
Dado 5 envios para "ana@exemplo.com" nas últimas 24 horas, o mais recente há 2 minutos
Quando o cliente pede um novo link para ela
Então a API responde `202` e nada é gravado
E o mesmo vale para "sem@exemplo.com", que não tem conta
```

### Cenário 9 — O login abre a sessão do web em dois cookies (caminho feliz, regras 10, 12 e 13)

```gherkin
Dado uma conta confirmada "ana@exemplo.com" com a senha "Clinica#2026"
Quando o web envia `POST /sessions` com essas credenciais, sem `Clinicore-Client`
Então a API responde `201` com `Location: /sessions/current` e o `user` sem `tokens`
E `Set-Cookie` traz `clinicore_access` com `Path=/`, `Max-Age=900`, `HttpOnly` e `SameSite=Lax`
E `Set-Cookie` traz `clinicore_refresh` com `Path=/sessions/current/tokens`, `Max-Age=86400`, `HttpOnly` e `SameSite=Lax`
E nenhum cookie tem `Domain`, e com `APP_ENV` `production` os dois têm `Secure`
E existe uma sessão `web` que vence em 24 horas
E `GET /sessions/current` com o cookie de acesso responde `200` com o mesmo `user`
```

### Cenário 10 — O login do app devolve os tokens no corpo (caminho feliz, regras 1 e 11)

```gherkin
Dado uma conta confirmada "ana@exemplo.com"
Quando o app envia `POST /sessions` com `Clinicore-Client: mobile`, sem `Origin` e sem `Cookie`
Então a API responde `201` sem `Set-Cookie`, com `user` e `tokens`
E `tokens.accessTokenExpiresIn` é 900 e o JWT traz `cli` "mobile"
E existe uma sessão `mobile` que vence em 7 dias
E `GET /sessions/current` com `Authorization: Bearer` e `Clinicore-Client: mobile` responde `200`
```

### Cenário 11 — Um transporte não autentica pelo outro (exceção, regra 1)

```gherkin
Dado uma sessão aberta pelo web e outra pelo app
Quando o cookie de acesso do web é mandado como `Authorization: Bearer` com `Clinicore-Client: mobile`
Então a API responde `401` `invalid-session`
E o refresh do app mandado no corpo sem a sessão ser `mobile`, ou o do web mandado pelo app, responde `401` `invalid-session`
E as duas sessões continuam existindo
E `Clinicore-Client: desktop` responde `400` `invalid-client`
```

### Cenário 12 — Senha errada e e-mail sem conta respondem igual (exceção, regra 8)

```gherkin
Dado uma conta confirmada "ana@exemplo.com"
Quando o cliente entra com "ana@exemplo.com" e a senha "Errada#2026"
E quando o cliente entra com "sem@exemplo.com" e qualquer senha
Então as duas respostas são `401` `invalid-credentials`, com o mesmo corpo e `WWW-Authenticate: Bearer realm="clinicore"`
E nenhuma sessão é criada
```

### Cenário 13 — Login sem confirmar o e-mail é recusado e reenvia o link (exceção, regras 9 e 13)

```gherkin
Dado uma conta não confirmada "ana@exemplo.com" sem envio no último minuto
Quando o cliente entra com a senha certa
Então a API responde `403` `email-not-verified` sem `Set-Cookie`
E a `email_outbox` ganha uma linha para o endereço
E repetir o login dentro de 60 segundos responde `403` sem nova linha na `email_outbox`
```

### Cenário 14 — O sexto login derruba a sessão mais antiga (exceção, regra 13)

```gherkin
Dado uma conta confirmada com 5 sessões, 3 do web e 2 do app
Quando ela entra de novo
Então ela tem 5 sessões, sem a mais antiga
E o access token da mais antiga responde `401` `invalid-session` na requisição seguinte
E o refresh dela responde `401` `invalid-session`
```

### Cenário 15 — O refresh do web rotaciona (caminho feliz, regra 12)

```gherkin
Dado uma sessão do web aberta há 2 horas
Quando o web envia `POST /sessions/current/tokens` com o cookie `clinicore_refresh`
Então a API responde `204` com dois `Set-Cookie` novos
E o `refresh_token_hash` da sessão mudou e o `expires_at` está 24 horas à frente
E o refresh do app, com `Clinicore-Client: mobile` e o `refreshToken` no corpo, responde `200` com `tokens` e empurra 7 dias
```

### Cenário 16 — Reusar o refresh antigo derruba a sessão (exceção, regra 12)

```gherkin
Dado uma sessão do web cujo refresh já foi rotacionado uma vez
Quando o web envia `POST /sessions/current/tokens` com o refresh anterior
Então a API responde `401` `session-reused`
E a sessão não existe mais
E o access token emitido na rotação responde `401` `invalid-session`
```

### Cenário 17 — O refresh não passa dos 30 dias (exceção, regra 12)

```gherkin
Dado uma sessão do app criada há 29 dias e 23 horas, renovada há 1 hora
Quando o app renova
Então a API responde `200` e o `expires_at` é o `created_at` mais 30 dias
Dado a mesma sessão com o `created_at` de 30 dias e 1 minuto atrás
Quando o app renova
Então a API responde `401` `invalid-session`
```

### Cenário 18 — Sair derruba a sessão na hora (caminho feliz, regras 10 e 12)

```gherkin
Dado uma sessão do web com o access token ainda válido
Quando o web envia `DELETE /sessions/current`
Então a API responde `204` com os dois cookies em `Max-Age=0`
E existe `auth:revoked:<sessionId>` no Redis com 7 dias de vida
E a sessão não existe mais
E o mesmo access token responde `401` `invalid-session` em `GET /sessions/current` (FR-ACC-12.1)
E repetir o `DELETE` responde `401` `invalid-session`
```

### Cenário 19 — O Redis fora fecha a porta (exceção, regras 12, 13 e 14)

```gherkin
Dado o Redis inalcançável
Quando o cliente chama `DELETE /sessions/current`, `POST /sessions` ou `POST /users`
Então a API responde `503` `service-unavailable`
E nenhuma sessão é apagada nem criada, e nenhum usuário é gravado
E `GET /health` responde `200`
```

### Cenário 20 — A guarda de origem aceita o web da lista e o app, e barra o resto (exceção, regra 2)

```gherkin
Dado `ALLOWED_ORIGINS` "http://localhost:3000"
Quando um `POST /sessions` chega com `Origin: http://localhost:3000`
Então a requisição segue
E com `Origin: http://localhost:3000.evil.example` a API responde `403` `invalid-origin`
E sem `Origin` e com `Cookie` a API responde `403` `invalid-origin`
E sem `Origin` e sem `Cookie` a requisição segue
E o `OPTIONS` de pré-voo de "http://localhost:3000" responde com `Access-Control-Allow-Origin: http://localhost:3000` e `Access-Control-Allow-Credentials: true`
E o de "https://outro.example" responde sem `Access-Control-Allow-Origin`
```

### Cenário 21 — Nenhuma resposta fica no cache (caminho feliz, regra 3)

```gherkin
Quando o cliente chama `GET /sessions/current`, `POST /users`, `DELETE /sessions/current`, `GET /health` e uma rota inexistente
Então toda resposta, de sucesso ou de erro, traz `Cache-Control: no-store`
```

### Cenário 22 — O limite por IP barra a sexta tentativa de login (exceção, regra 14)

```gherkin
Dado 5 `POST /sessions` do mesmo IP nos últimos 60 segundos
Quando chega o sexto do mesmo IP
Então a API responde `429` `rate-limited` com `Retry-After`
E um `POST /sessions` de outro IP segue
```

### Cenário 23 — Dez falhas travam o e-mail por 15 minutos (exceção, regra 14)

```gherkin
Dado 10 logins com senha errada para "ana@exemplo.com" em 15 minutos, de IPs diferentes
Quando ela entra com a senha certa
Então a API responde `429` `rate-limited`
E o login de "bia@exemplo.com" segue
E um login com a senha certa não soma no contador
```

### Cenário 24 — A fonte de IP não deixa o cliente escapar do limite (exceção, regra 4)

```gherkin
Dado `CLIENT_IP_SOURCE` "x-forwarded-for:1"
Quando 6 `POST /sessions` chegam com `X-Forwarded-For` "1.1.1.<n>, 203.0.113.7", cada um com um `n` diferente
Então o sexto responde `429`, porque todos contam como "203.0.113.7"
Dado `CLIENT_IP_SOURCE` "cf-connecting-ip"
Quando 6 chegam com `CF-Connecting-IP` "203.0.113.7" e `X-Forwarded-For` diferentes
Então o sexto responde `429`
E requisições sem o header dividem o contador `unknown`
E dois IPv6 do mesmo `/64` dividem um contador
```

### Cenário 25 — O worker envia e apaga a linha (caminho feliz, regra 16)

```gherkin
Dado uma linha `pending` na `email_outbox` que vence em 1 hora
Quando o worker roda um ciclo da fila
Então o SMTP de teste recebe um e-mail para o endereço, de `MAIL_FROM`, com o assunto e os dois corpos da linha
E a linha não existe mais
```

### Cenário 26 — O SMTP falha, a linha volta com espera, e a quinta falha desiste (exceção, regra 16)

```gherkin
Dado o SMTP de teste recusando todo envio, e uma linha `pending` com `attempts` 0
Quando o worker roda um ciclo
Então a linha tem `attempts` 1 e `next_attempt_at` 1 minuto à frente
E o log traz `warn` "mail delivery failed, retrying" com o endereço mascarado
Dado a mesma linha com `attempts` 4 e `next_attempt_at` vencido
Quando o worker roda um ciclo
Então a linha é `failed`, com `html_body` e `text_body` nulos
E o log traz `error` "mail delivery gave up"
```

### Cenário 27 — Linha vencida não sai, e dois workers não dividem uma linha (caminho alternativo, regra 16)

```gherkin
Dado uma linha `pending` com `expires_at` 1 segundo atrás
Quando o worker roda um ciclo
Então nada é enviado e a linha continua `pending`
Dado 10 linhas `pending` e dois workers reservando ao mesmo tempo
Então cada linha é reservada por um worker só
E uma linha reservada há 2 minutos e 1 segundo, sem resultado, volta a ser reservada
```

### Cenário 28 — O worker para sem perder o envio em curso (caminho alternativo, regra 16)

```gherkin
Dado o worker enviando uma linha
Quando ele recebe `SIGTERM`
Então termina o envio, apaga a linha e sai com código 0
E não reserva nenhuma linha depois do sinal
```

### Cenário 29 — A purga apaga o que venceu, e repetir apaga zero (caminho feliz, regra 17)

```gherkin
Dado uma sessão vencida, uma criada há 31 dias e uma válida
E uma verificação vencida e uma válida
E um envio registrado há 25 horas e um há 23 horas
E uma linha da `email_outbox` vencida e uma válida
Quando a purga roda
Então ela apaga 2 sessões, 1 verificação, 1 envio e 1 linha da fila, e mantém as válidas
E o log traz `info` "purged the expired authentication records" com essas quantidades
E rodar de novo apaga 0 de cada
```

### Cenário 30 — A próxima purga é a próxima 03:00 de São Paulo (caminho feliz, regra 17)

```gherkin
Quando a próxima execução é calculada a partir de `2026-11-20T02:59:59-03:00`
Então ela é `2026-11-20T03:00:00-03:00`
E a partir de `2026-11-20T03:00:00-03:00` ela é `2026-11-21T03:00:00-03:00`
```

### Cenário 31 — Ambiente incompleto não sobe (exceção, regra 18)

```gherkin
Dado o ambiente da API sem `JWT_SECRET` e com `APP_ORIGIN` fora de `ALLOWED_ORIGINS`
Quando a API sobe
Então ela escreve "Invalid environment:", uma linha para `APP_ORIGIN` e outra para `JWT_SECRET`, em ordem alfabética, sem nenhum valor
E sai com código 1 antes do bind
E o worker sem `SMTP_PASSWORD`, ou com `SMTP_TLS` "none" e `APP_ENV` "production", faz o mesmo antes de abrir conexão
E a API sobe sem nenhuma variável `SMTP_*` nem `MAIL_FROM`
```

### Cenário 32 — Nenhum segredo vai para o log (caminho feliz, regra 19)

```gherkin
Quando o cliente cria a conta, confirma, entra, renova e sai
Então nenhuma linha do log contém a senha, o segredo do link, o refresh token, o access token, o header `Cookie` ou o `Authorization`
```

### Cenário 33 — O schema e as consultas não divergem (caminho feliz, regra 19)

```gherkin
Dado as migrations aplicadas num banco vazio
Quando o CI roda `cargo sqlx prepare --workspace --check`
Então ele termina com código 0
```

---

## Impacto na arquitetura

- **[`apps/api`](../../../system/architecture/building-blocks/api.md):** os blocos `users`,
  `email_verifications`, `email_outbox`, `sessions` e `credentials` passam a existir; o encanamento HTTP
  ganha `http/client.rs` (o `Clinicore-Client`), `http/origin.rs`, `http/client_ip.rs`,
  `http/rate_limit.rs` e a camada de `Cache-Control`. A tabela "Rotas da autenticação" deixa de ser o
  contrato apagado e passa a ser o desta spec; as rotas de Google saem dela até a #7.
- **[worker](../../../system/architecture/building-blocks/worker.md):** nasce como está descrito, e o
  "envia e-mail de verdade em dev" passa a enviar só para o `MAIL_REDIRECT_TO`.
- **[Redis](../../../system/architecture/building-blocks/redis.md):** sem mudança de papel; as chaves de
  limite passam a ser as da regra 14.
- **[Ciclo de vida da sessão](../../../system/architecture/runtime/session-lifecycle.md)** e
  **[Cadastro e confirmação de e-mail](../../../system/architecture/runtime/sign-up-and-email-verification.md):**
  sem mudança: os dois já descrevem a ordem das regras 13 e 14 e o limite da confirmação por IP e por
  faixa de rede.
- **[Segurança](../../../system/architecture/concepts/security.md):** o `Secure` segue o `APP_ENV`, não a
  `API_URL`; o IP do cliente vem de `CLIENT_IP_SOURCE`.
- **[Configuração](../../../system/architecture/concepts/configuration.md):** o worker passa a ler dez
  variáveis, com `SMTP_TLS` e `MAIL_REDIRECT_TO`; a API ganha as cinco da regra 18.
- **[Implantação](../../../system/architecture/deployment.md):** o Mailpit entra no compose de dev, e o
  worker roda local por `cargo run -p clinicore-worker`.

## Impacto no modelo de dados

As entidades `User`, `Account`, `Session`, `Verification`, `EmailDispatch` e `EmailOutbox` já estão no
[modelo de dados](../../../system/data-model/README.md). Esta entrega as cria como tabelas, e o modelo
físico volta com a primeira migration. O `image` do `User` sai: a #7 decide se o Google o traz.

| Tabela | Colunas |
| --- | --- |
| `users` | `id` `uuid` PK; `name` `text` não nulo; `email` `text` não nulo, único, `CHECK (email = lower(email))`; `email_verified` `boolean` não nulo, padrão `false`; `created_at` e `updated_at` `timestamptz` não nulos |
| `accounts` | `id` `uuid` PK; `user_id` → `users.id` `ON DELETE CASCADE`; `provider` `account_provider` não nulo; `provider_account_id` `text` nulo; `password_hash` `text` nulo, `CHECK (provider <> 'credential' OR password_hash IS NOT NULL)`; `created_at` e `updated_at`; únicos em (`user_id`, `provider`) e em (`provider`, `provider_account_id`) |
| `sessions` | `id` `uuid` PK; `user_id` → `users.id` `ON DELETE CASCADE`; `client` `session_client` não nulo; `refresh_token_hash` `bytea` não nulo, único; `expires_at` `timestamptz` não nulo; `ip_address` `inet` nulo; `user_agent` `text` nulo, até 512 caracteres; `created_at` e `updated_at` |
| `verifications` | `id` `uuid` PK; `email` `text` não nulo, em minúsculas; `purpose` `verification_purpose` não nulo; `token_hash` `bytea` não nulo, único; `expires_at` `timestamptz` não nulo; `consumed_at` `timestamptz` nulo; `created_at` |
| `email_dispatches` | `id` `uuid` PK; `email` `text` não nulo, em minúsculas; `kind` `email_dispatch_kind` não nulo; `created_at` |
| `email_outbox` | as colunas, o `CHECK` e os índices da [spec 006](006-worker-e-fila-de-email-api.md#data-model-impact), sem mudança |

Os tipos e os índices são os de [`storage.md`](../../../system/data-model/storage.md). O
`account_provider` já nasce com `google`, e o `verification_purpose` e o `email_dispatch_kind` com
`password_reset`, para a #7 não precisar de migration de tipo. Os padrões de acesso usados são os de
[`access-patterns.md`](../../../system/data-model/access-patterns.md), menos os de redefinição e troca de
senha.

## Fora de escopo

- **Google, recuperação e troca de senha** (`FR-ACC-09`, `10` e `11`): a #7, sobre estas tabelas.
- **Passkey** (`FR-ACC-13` e `14`): o M3.
- **As telas** do web, do Android e do iOS: as #8, #9 e #10.
- **Papéis, clínicas e tenancy**: outro épico; nenhuma rota desta entrega conhece clínica.
- **Apagar a conta**: nenhum requisito do M1 pede, e o modelo já diz como seria.
- **O deploy**: Cloudflare Tunnel, firewall e a regra de não publicar a porta da API são da
  hospedagem, que não existe até o fim do desenvolvimento. Esta entrega só lê `CLIENT_IP_SOURCE`.
- **O provedor de e-mail de produção**: decisão do lançamento. O worker fala SMTP com qualquer um.
- **O `GET /health` conferir o Redis e o Postgres**: continua respondendo que o processo está de pé.

## Quebra em tarefas

As quatro tarefas são etapas de commit da mesma issue #6, numa branch e num PR. Nenhuma vira sub-issue.

| # | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- |
| 1 | Configurar a API e o worker, ligar o Redis e criar as tabelas | `crates/core` (config, Redis, `lifetimes.rs`), migrations, `.sqlx/`, `.env.example`, compose com Mailpit, CI da API com Redis, `http/` (cliente, origem, CORS, `Cache-Control`, IP do cliente) | Cenários 31 e 33 | — |
| 2 | Criar a conta e confirmar o e-mail | `credentials/`, `users/`, `email_verifications/`, `email_outbox.rs`, templates no `core`, `http/rate_limit.rs` | Cenários 1 a 8, 20, 21 e 24 | 1 |
| 3 | Enviar o e-mail e purgar o que venceu no worker | `crates/worker/` (`main.rs`, `outbox.rs`, `mail.rs`, `schedule.rs`, `purge.rs`) e os testes dele | Cenários 25 a 30 | 2 |
| 4 | Entrar, renovar e sair, no web e no app | `sessions/` com `tokens/`, a denylist, os limites da sessão | Cenários 9 a 19, 22, 23 e 32 | 2 |

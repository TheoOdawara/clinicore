# Segurança

## Autenticação e sessão — existe

- **A senha é guardada como hash Argon2**, e a política de senha forte é conferida no cadastro, na
  redefinição e na troca.
- **Redefinir a senha derruba todas as sessões do usuário, e trocá-la derruba todas menos a que fez o
  pedido.** A redefinição também dá login por senha a quem só entrou com Google.
- **A sessão é uma linha no PostgreSQL**, com o hash do refresh token. O access token é um JWT HS256 de
  15 minutos; o refresh token vive 24 horas no web e 7 dias no app, é rotacionado a cada uso, e o reuso
  de um antigo derruba a sessão. Cada usuário tem no máximo 5 sessões, e nenhuma passa de 30 dias.
- **A revogação é imediata**, pela denylist no Redis, escrita antes de a sessão ser apagada
  ([ADR 0012](../../decisions/0012-revogacao-de-sessao-antes-de-apagar.md)).
- **O transporte do token depende do cliente.** O web recebe cookie `HttpOnly` em `SameSite=Lax`, e o
  `Secure` segue o esquema da `API_URL`; o app recebe o token no corpo e o manda no `Authorization`.
- **A origem é conferida em toda rota**, contra `ALLOWED_ORIGINS`, e o CORS só libera essas origens.
- **Toda rota limitada conta em duas chaves**: o IP e uma identidade — a sessão, o e-mail ou a faixa de
  rede.
- **Nenhuma resposta revela se um e-mail tem conta**: cadastro, reenvio e pedido de redefinição
  respondem `202` sempre, e o login confere a senha mesmo sem conta.

O passo a passo está em [`../runtime/session-lifecycle.md`](../runtime/session-lifecycle.md).

## A construir

| O que | Requisito | Marco |
| --- | --- | --- |
| Token do app guardado no Keychain e no Keystore | `NFR-SEC-04` | M1 · #98 |
| Login com Google | `FR-ACC-09` | M1 · #120 |
| Isolamento entre clientes: todo dado de clínica pertence a um cliente | `NFR-SEC-01` | M1 |
| Papéis com menor privilégio e alcance por clínica dentro da rede | `FR-ACC-03`, `FR-ACC-04`, `NFR-SEC-03` | M1 |
| Trilha de auditoria de acesso ao prontuário | `FR-ACC-06` | M1 |
| Volumes e backups cifrados | `NFR-SEC-02` | M1, com a hospedagem |
| Dado de saúde no aparelho: banco cifrado, expiração em 72 horas sem sincronizar e limpeza no logout | `NFR-SEC-04` | M1 |
| Login com passkey | `FR-ACC-13`, `FR-ACC-14` | M2 · #100 |

## Segredos

Nenhum valor real fica no repo nem em arquivo no disco; ver [configuration.md](configuration.md).

## Dívida conhecida

Os achados de segurança adiados são issues com a label `security-debt`, listadas em
[`../risks.md`](../risks.md).

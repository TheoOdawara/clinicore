# Armazenamento

O modelo físico: PostgreSQL 18, com o schema aplicado pelas migrations de `apps/api/migrations/`.

## Tabelas

As tabelas, as colunas, as chaves e as restrições estão no [modelo físico](README.md#modelo-físico).
Toda chave é `uuid` com `gen_random_uuid()` como padrão, e toda data é `timestamptz`. Os `CHECK`
não aparecem no diagrama: `email = lower(email)` em `users` e em `email_outbox`, o de `accounts` que
exige `password_hash` na credencial, e os dois da `email_outbox`: `attempts` entre 0 e 5, e os dois
corpos preenchidos se e somente se a linha é `pending`.

## Tipos

| Tipo | Valores |
| --- | --- |
| `account_provider` | `credential`, `google` |
| `verification_purpose` | `email_verification`, `password_reset` |
| `email_dispatch_kind` | `email_verification`, `password_reset` |
| `session_client` | `web`, `mobile` |
| `email_outbox_status` | `pending`, `failed` |

## Índices

Além dos que as chaves e as restrições de unicidade criam:

| Índice | Serve a |
| --- | --- |
| `sessions (user_id)` | o teto de 5 sessões por usuário |
| `sessions (expires_at)` | a purga |
| `verifications (expires_at)` | a purga |
| `verifications (email, purpose, consumed_at)` | consumir, na confirmação, todas as verificações pendentes do endereço |
| `email_dispatches (created_at)` | a purga |
| `email_dispatches (email, kind, created_at)` | a contagem do limite de envio |
| `email_outbox (next_attempt_at) WHERE status = 'pending'` | a reserva do worker |
| `email_outbox (expires_at)` | a purga |

## Migrations

- **Uma migration é um arquivo SQL** em `apps/api/migrations/`, aplicado por `sqlx migrate run`. Hoje há
  duas, `20260929120000_init.sql` e `20261021120000_email_outbox.sql`.
- **Depois de mudar o schema, o `.sqlx/` é regerado** com `cargo sqlx prepare --workspace`, e o CI
  reprova a consulta que divergiu.
- **Os testes criam um banco por teste** a partir das mesmas migrations.

## Fora do PostgreSQL

| Onde | O que | Vida |
| --- | --- | --- |
| Redis | contadores do limite de requisições | a janela do limite |
| Redis | `auth:revoked:<sessão>`, a denylist de revogação | 7 dias |
| Aparelho, no M1 | banco local cifrado do app, com o dado offline e a fila de escrita | 72 horas sem sincronizar |

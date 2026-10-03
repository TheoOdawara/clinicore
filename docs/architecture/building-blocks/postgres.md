# PostgreSQL 18

O único armazenamento durável do sistema. Só a API fala com ele, por sqlx, e o schema é o que as
migrations de `apps/api/migrations/` aplicam.

**Nunca recebe:** conexão de outro app, SQL montado em runtime, migration aplicada à mão.

## O que ele garante

- **Unicidade e integridade são constraints**, não código de aplicação: `UNIQUE`, `CHECK` e chave
  estrangeira com `ON DELETE CASCADE`.
- **Concorrência se resolve aqui**: `ON CONFLICT … DO NOTHING` para unicidade e
  `pg_advisory_xact_lock` para serializar por chave, dentro da transação.
- **As consultas são conferidas na compilação.** O `query!` do sqlx valida cada consulta contra o
  schema, e o `.sqlx/` commitado deixa o CI reprovar uma consulta que divergiu das migrations.

## Para mudar com segurança

Mudança de schema é migration nova, seguida de `cargo sqlx prepare --workspace` para regerar o
`.sqlx/`. As tabelas, as chaves e os índices estão em [`../../data-model/`](../../data-model/README.md).

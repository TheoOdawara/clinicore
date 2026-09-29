# 0009. API em Rust, com axum e sqlx, num workspace por camada

- Status: superseded by 0010 na organização do workspace, no erro e nos códigos de campo; a troca de
  stack segue valendo
- Date: 2026-09-28
- Substitui: `0001-api-em-nestjs-typeorm-e-redis.md`
- Emenda: a seção Arquitetura do `CLAUDE.md` da raiz e o `apps/api/CLAUDE.md`, reescritos na
  migração (spec `005-migrar-api-para-rust.md`)

## Context

O `apps/api` está em NestJS 11 desde a ADR 0001. São cerca de 8,6 mil linhas de TypeScript: 2 mil de
estrutura transversal (`core/`, `common/`, boot) e a feature `auth` completa, com 13 rotas e o
`/health`. Nada está em produção, e nenhum dado precisa ser preservado.

O mantenedor quer manter a API em Rust daqui em diante. É uma preferência declarada, e é ela que abre
esta decisão; não houve falha técnica do NestJS que a force.

O que precisa continuar valendo:

- **O contrato HTTP.** O `apps/web` chama a API em origem cruzada, e o `apps/mobile` gera o cliente
  do OpenAPI. Rotas, status, corpos, cookies, headers e limites são os das specs 002, 003 e 004.
- **O REST orientado a recurso e o Problem Details da ADR 0006.**
- **As camadas `rota → regra → persistência`.** No NestJS, quem as fazia valer era o container de
  DI. Num crate Rust único a privacidade de módulo não faz isso: um handler alcança o repository da
  própria feature.
- **Toda variável de ambiente obrigatória**, validada no boot, sem default.

## Decision

O `apps/api` é reescrito em **Rust**, num **Cargo workspace com um crate por camada**:

| Crate | Tipo | Depende de | Contém |
| --- | --- | --- | --- |
| `http` | binário | `app` | boot, `Router`, handlers, extractors, DTO, OpenAPI, `ApiError` → Problem Details, telemetria |
| `app` | biblioteca | `infra` | services, domínio, `AppError` |
| `infra` | biblioteca | — | configuração, repositories, Postgres, Redis, e-mail |

O `http` não depende do `infra`, então um handler não compila se chamar um repository. O `PgPool` não
é `pub` no `infra`, então um service não abre consulta. **O compilador faz o papel que o container de
DI fazia.** As features são módulos dentro de cada crate, nunca um crate por feature.

| Sai | Entra |
| --- | --- |
| NestJS 11 sobre Express | axum sobre tokio, com as camadas do `tower-http` |
| TypeORM 1.1 com `@nestjs/typeorm` | sqlx, com `query!` e `query_as!` checados contra o schema em compile time |
| Migrations do TypeORM | `sqlx migrate`, em `migrations/`, com uma migration inicial equivalente às três atuais |
| `class-validator` e `class-transformer` | `serde` e `validator` |
| `@nestjs/swagger` | `utoipa` |
| Pino atrás de um `LoggerService` | `tracing` e `tracing-subscriber`, com o `TraceLayer` do `tower-http` no log de requisição |
| `@nestjs/config` validado por `class-validator` | leitura e validação próprias em `infra`, sem default |
| `@nestjs/passport`, `@nestjs/jwt`, `@node-rs/argon2` | `jsonwebtoken` e `argon2` |
| `passport-google-oauth20` | `reqwest` contra os endpoints do Google |
| `@nestjs/throttler` com storage em Redis | limite por IP em Redis, escrito no crate `http` sobre o `redis` |
| `ioredis` | `redis` |
| Nodemailer | `lettre` |
| `@nestjs/terminus` | handler próprio em `/health` |
| Jest, `ts-jest`, supertest, `@nestjs/testing` | `cargo test`, `#[sqlx::test]`, `tower::ServiceExt::oneshot`, `wiremock` |
| ESLint, `typescript-eslint`, Prettier | `cargo clippy -- -D warnings` e `cargo fmt` |
| Node 26 e npm | Rust 1.98.1, edition 2024, fixado em `rust-toolchain.toml` |

Decisões de detalhe:

- **A regra "raw SQL proibido" é emendada.** Ela existia porque o SQL do TypeORM não tem tipo. O
  `query!` do sqlx confere a consulta contra o schema na compilação, então o SQL passa a ser escrito
  à mão. Ele fica só nos repositories do crate `infra` e sempre passa por `query!` ou `query_as!`. SQL
  montado por `format!` ou concatenação é proibido. O `.sqlx/` é commitado, e o CI roda
  `cargo sqlx prepare --check`.
- **`NODE_ENV` vira `APP_ENV`**, com o mesmo domínio: `development`, `production` e `test`.
- **`LOG_LEVEL` passa a aceitar `error`, `warn`, `info`, `debug`, `trace` e `off`**, que é o domínio
  do `tracing`. O `silent` do Pino vira `off`.
- **O log pode mudar de formato.** É JSON em `stdout` fora de `development` e legível em
  `development`. Registra método, path, status e duração. O `/health` não é logado. Header, cookie,
  senha e connection string nunca são registrados, porque o layer de requisição não lê header.
- **O teste de integração usa `#[sqlx::test]`**, que cria um banco novo por teste. A suíte deixa de
  precisar rodar em série.

## Consequences

Fica mais fácil:

- As camadas passam a ser verificadas pelo compilador, e não mais por um container em runtime.
- A consulta errada contra o schema falha na compilação, e não mais no teste ou em produção.
- Os testes rodam em paralelo, cada um com o seu banco.
- As armadilhas do Nest 11 somem: ESM-only do 12, `ConfigService.get` devolvendo `any`, o
  `forRoot` assíncrono e o `reflect-metadata`.

Fica mais difícil, e é aceito:

- **O que o Nest dava pronto vira código escrito**: rate limit, guard de `Origin`, montagem do
  Problem Details, validação de ambiente e o fluxo OAuth do Google.
- **O Rust não tem par oficial do `@nestjs/bullmq`.** A #71, que é o primeiro job, escolhe a fila e o
  agendador na spec dela.
- **O `http` precisa de um newtype `ApiError(AppError)`** para implementar o `IntoResponse`, por causa
  da orphan rule.
- **O código de campo da validação é escrito à mão** em cada `#[validate]`. Os códigos que o web e o
  mobile já conhecem (`IS_EMAIL`, `WEAK_PASSWORD` e os outros) precisam sair iguais.
- **Os dois clientes seguem acoplados ao OpenAPI.** O cliente Dart gerado do documento do `utoipa`
  precisa sair idêntico ao gerado do `@nestjs/swagger`.
- A API muda de ecossistema enquanto `apps/web` e `apps/site` seguem em Node. O repo já é de apps
  independentes, e a fronteira continua sendo o contrato HTTP.

## Alternatives considered

- **Manter o NestJS** · rejeitada: o mantenedor quer a API em Rust, e nada está em produção. Este é o
  momento mais barato da troca.
- **SeaORM** · rejeitada: preserva a regra de não escrever SQL, mas é um ORM com entity e query builder
  e erra em runtime, que é o problema que motivou a regra.
- **Diesel** · rejeitada: o query builder é tipado, mas a base é síncrona (async só pelo
  `diesel-async`) e a curva é maior. O sqlx é o padrão atual para serviço async sobre tokio.
- **Crate único com módulos por feature** · rejeitada: as camadas voltam a ser convenção, e o crate
  inteiro recompila a cada mudança, o que piora a cada épico do produto.
- **Um crate por feature** · rejeitada por ora: são crates demais para uma feature só. Reabre por ADR
  própria quando o tempo de compilação medido pedir.
- **Migração por proxy, com as duas APIs no ar** · rejeitada: nada está em produção, então a
  convivência se resolve com `apps/api-rs` ao lado até a paridade, sem roteamento.
- **Manter o Pino como formato de log** · rejeitada: o log não é contrato HTTP, e imitar o formato do
  Pino seria um formatter próprio sem ganho.

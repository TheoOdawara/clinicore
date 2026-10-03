# 005 — Migrar a API para Rust e extrair o scaffold NestJS

> **Status:** publicada
> **Perfil:** API
> **Módulo:** `apps/api` (durante a migração, `apps/api-rs`), `.github/workflows/ci.yml`, e fora do repo
> `~/Projects/nestjs-scaffold` e `~/Projects/nestjs-scaffold-auth`
> **Epic:** #114, dentro da #1 — Plataforma
> **Decisões:** `docs/decisions/0009-api-em-rust-com-axum-e-sqlx.md` ·
> `docs/decisions/0010-api-rs-com-crate-por-processo.md` ·
> `docs/decisions/0006-api-rest-e-problem-details.md`
> **Oráculo de comportamento:** `docs/specs/002-scaffold-apps-e-ci.md`,
> `docs/specs/autenticacao/003-autenticacao-api.md` e
> `docs/specs/autenticacao/004-sessao-por-token-api.md`. Esta spec não muda nenhum comportamento
> delas; as regras daqui citam as de lá por número.

## Acceptance Criteria

### Contrato

Nenhuma rota nova, nenhuma removida. São as 13 rotas que existem hoje, com o comportamento das specs
de origem:

| Método | Rota | Auth / Role | Idempotente | Origem |
| --- | --- | --- | --- | --- |
| `GET` | `/health` | público | Sim | 002, com o corpo `{"status":"ok"}` |
| `POST` | `/users` | público | Não | 003, regras 4, 5, 8 |
| `POST` | `/email-verifications` | público | Não | 003, regras 5, 15 |
| `POST` | `/email-verifications/confirmation` | público | Não | 003, regra 5 |
| `POST` | `/sessions` | público | Não | 003, regras 2, 16 · 004, regra 1 |
| `GET` | `/sessions/current` | sessão | Sim | 003, regra 2 · 004, regra 2 |
| `DELETE` | `/sessions/current` | sessão | Sim | 003, regra 2 · 004, regra 2 |
| `POST` | `/sessions/current/tokens` | refresh por cookie, ou no corpo com `Clinicore-Client: mobile` | Não | 003, regra 3 · 004, regras 2, 3 |
| `POST` | `/password-resets` | público | Não | 003, regras 7, 15 |
| `POST` | `/password-resets/confirmation` | público | Não | 003, regra 7 |
| `PUT` | `/users/me/password` | sessão | Sim | 003, regra 7 |
| `GET` | `/oauth/google` | público | Não | 003, regra 6 |
| `GET` | `/oauth/google/callback` | público | Não | 003, regra 6 |

Fora de produção, a API também serve a UI do OpenAPI em `GET /api` e o documento em `GET /api-json`,
como hoje.

### Request

Idêntica, campo a campo, às specs de origem: os mesmos nomes, tipos, obrigatoriedades e limites. O
`errors[].code` de cada campo é o do `validator` (`email`, `length`, `name`, `weak_password`), e campo
desconhecido ou de tipo errado sai como `invalid`, sempre em `400 VALIDATION_FAILED` (003, Cenário 12).

### Response

Idêntica às specs de origem: status, corpo, `Set-Cookie`, headers e o Problem Details da ADR 0006,
com `type` em `tag:clinicore.com.br,2026:<código-em-kebab>` para o catálogo e `about:blank` para o
resto, `title` fixo por `type`, e `Content-Type: application/problem+json`.

| Status | Quando |
| --- | --- |
| os das specs de origem | nas mesmas condições que as de lá |

### Perfis e privilégios

| Papel | Permissão | Observação |
| --- | --- | --- |
| Os das specs 003 e 004 | — | A migração não cria papel nem permissão |

---

## Regras de Negócio

### 1. Contrato observável

- Toda rota das specs de origem responde na API Rust com a mesma URL, o mesmo status, os mesmos
  cookies com os mesmos atributos e os mesmos headers de contrato. O corpo segue o Problem Details
  da ADR 0006; os códigos de campo em `errors[].code` são os do Rust (seção Request).
- Os limites continuam iguais: os por IP da regra 11 da 003 e da regra 8 da 004, o limite por
  endereço da regra 15 da 003, e o teto de 5 sessões da regra 16 da 003.
- **O IP do cliente muda de mecanismo.** `TRUSTED_PROXIES` e o percurso do `trust proxy` do Express
  saem; entra `CLIENT_IP_SOURCE`, lida pelo `axum-client-ip`: `ConnectInfo` sem proxy, ou o header que
  o proxy escreve. O Cenário 26 da 003 é portado com `RightmostXForwardedFor`, e o IP contado é o
  último salto do `x-forwarded-for`, o que o proxy acrescentou.
- **Validação:** cada cenário Gherkin das specs 002, 003 e 004 cuja rota existe hoje é portado para um
  teste de integração em `crates/app/tests/` e passa contra a API Rust. Ficam de fora o Cenário 34 da
  003 (#71) e as regras 5 a 9 da 004 (#94 e #100), que não estão implementadas.
- Cada cenário de origem pertence à primeira task em que todas as rotas que ele chama existem. O
  mecanismo é construído onde o escopo da task diz; o teste de ponta a ponta fecha na task dona do
  cenário.
- Cinco cenários de origem são adaptados ao Rust:
  - O Cenário 4 da 002 e o Cenário 35 da 003 provam o TypeORM e são substituídos pelos Cenários 1 e 7
    desta spec: o schema vem só da migration sqlx, e o `.sqlx/` é conferido contra as consultas.
  - O Cenário 29 da 003 é portado sem a linha do worker, que é a #71.
  - **A regra 17 da 003 roda dentro da API, sem fila.** O `purge::run` apaga o que venceu ao subir e
    a cada hora, em toda réplica, porque o `DELETE` repetido apaga zero linhas. A sessão passada
    do teto de 30 dias também é apagada, porque nenhum refresh a renova. O Cenário 33 é portado
    chamando o `purge_expired` direto; o 34 é da #71.
  - O Cenário 28 da 003 confere a causa no log em nível `error` pelo `tracing`, no lugar da stack no
    log do Pino.

### 2. O OpenAPI gera o cliente com as mesmas operações

- O documento servido em `/api-json` pela API Rust, dado ao `swagger_parser` do `apps/mobile`, gera
  `apps/mobile/lib/shared/api/` com as mesmas rotas, métodos, status e campos da API NestJS.
- **O diff do cliente é só o que as ADRs 0010 e 0011 decidiram:** schemas com o nome do tipo Rust, o
  método com o nome do handler Rust (`signIn` no lugar de `authControllerSignIn`), o modelo `Problem`,
  o enum `ClinicoreClient`, o `tokens` opcional do sign-in e o corpo opcional do refresh, que o web não
  manda. Qualquer outra linha do diff é regressão de contrato. As rotas de `/password-resets` somem até
  a #119 portá-las.
- **A #114 já regenerou o cliente contra a API Rust**, num arquivo só (`lib/shared/api/api.dart`), e a
  task 7 regenera de novo com as rotas das tasks seguintes.
- **Validação:** a task 7 regenera o cliente a partir da API Rust, revisa o diff de
  `apps/mobile/lib/shared/api` contra a lista acima e ajusta o código do `apps/mobile` que usa os nomes
  antigos, com os gates do Flutter verdes.

### 3. Toda variável de ambiente é obrigatória, tipada e validada no boot

- As 17 variáveis são lidas uma vez, no boot, para tipos que só existem com valor válido. Não há
  default em ponto nenhum.
- `NODE_ENV` passa a se chamar `APP_ENV`. `LOG_LEVEL` troca `fatal` e `silent` por `off`.
- Com uma ou mais inválidas ou ausentes, o processo escreve em `stderr` a linha `Invalid environment:`
  seguida de uma linha por variável recusada, em ordem alfabética, no formato
  `  <NOME>: <formato esperado>`, e sai com código `1`, sem abrir a porta.

| Variável | Formato esperado (texto literal da linha) |
| --- | --- |
| `ALLOWED_ORIGINS` | "expected a comma-separated list of absolute URLs with no trailing slash (https://…)" |
| `API_URL` | "expected an absolute URL with no trailing slash (https://…)" |
| `APP_ENV` | "expected one of development, production, test" |
| `APP_ORIGIN` | "expected an absolute URL with no trailing slash (https://…)" |
| `CLIENT_IP_SOURCE` | "expected one of: CfConnectingIp, CloudFrontViewerAddress, ConnectInfo, FlyClientIp, RightmostXForwardedFor, TrueClientIp, XEnvoyExternalAddress, XRealIp" |
| `DATABASE_URL` | "expected a PostgreSQL connection string (postgresql://…)" |
| `GOOGLE_CLIENT_ID` | "expected a non-empty string" |
| `GOOGLE_CLIENT_SECRET` | "expected a non-empty string" |
| `JWT_SECRET` | "expected a string with at least 32 characters" |
| `LOG_LEVEL` | "expected one of: error, warn, info, debug, trace, off" |
| `MAIL_FROM` | "expected an email address equal to SMTP_USER" |
| `PORT` | "expected an integer between 1 and 65535" |
| `REDIS_URL` | "expected a Redis connection string (redis://…)" |
| `SMTP_HOST` | "expected a hostname" |
| `SMTP_PASSWORD` | "expected a non-empty string" |
| `SMTP_PORT` | "expected an integer between 1 and 65535" |
| `SMTP_USER` | "expected an email address" |

### 4. O log

- O log sai em `stdout` pelo `tracing`, em JSON quando `APP_ENV` não é `development`, e legível quando é.
- Cada requisição gera uma linha com método, path, status e duração em milissegundos, e o status é o
  final, venha ele do handler, de uma camada do `tower` ou de rota inexistente.
- `GET /health` não gera linha.
- Nenhuma linha contém o valor de `Authorization`, de `Cookie`, de `Set-Cookie`, de senha, de token
  ou de connection string.
- Um `500` registra a causa em nível `error`; o corpo de resposta continua sem detalhe (003, regra 10).

### 5. As camadas e o SQL

- O workspace tem dois crates, `core` e `app`, e o `app` depende do `core` (ADR 0010).
- SQL só existe no `queries.rs` de cada feature do `app`, e só por `query!` ou `query_as!`. SQL
  montado por `format!` ou concatenação é proibido. Transação só no `queries.rs`.
- O `.sqlx/` é commitado, e o CI roda `cargo sqlx prepare --workspace --check`.

### 6. A convivência e a troca

- Durante as tasks 2 a 4 o Rust viveu em `apps/api-rs`, e o NestJS ficou em `apps/api`, congelado e sem
  gate de CI.
- A task 2 troca o job `api` do CI pelo job `api-rs`, que roda `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, `sqlx migrate run`, `cargo sqlx prepare --workspace --check`
  e `cargo test`, com Postgres 18 e Redis 8 como services.
- A task 7 cria a tag `api-nestjs-final` no último commit em que `apps/api` é NestJS, apaga o NestJS e
  renomeia `apps/api-rs` para `apps/api`. O job volta a se chamar `api`.
- **A task 7 veio antes das tasks 5 e 6**, por decisão do dono do produto em 2026-10-03: o NestJS
  congelado contaminava o contexto de quem trabalha no Rust. As rotas das tasks 5 e 6 deixam de existir
  no repositório até serem escritas em Rust, já em `apps/api`; o comportamento delas segue descrito na
  spec 003, e o código antigo fica na tag.
- A porta continua :3333.

### 7. O scaffold NestJS

- O NestJS de hoje é copiado para duas pastas fora do repo, sem git:
  - `~/Projects/nestjs-scaffold`: boot, configuração validada, logger e log de requisição, Problem
    Details, TypeORM, Redis, e-mail, `/health`, guard de `Origin`, rate limit, ESLint, Prettier,
    TypeScript, Jest e o `compose.yaml` de Postgres e Redis. Sem `features/auth`.
  - `~/Projects/nestjs-scaffold-auth`: tudo isso mais `features/auth` completa.
- Nenhuma das duas contém o nome do projeto. `clinicore` vira `app`: o cookie `clinicore_refresh`
  vira `app_refresh`, o header `Clinicore-Client` vira `App-Client` e o `type` passa a
  `tag:example.com,2026:<código>`.
- Cada pasta leva um `CLAUDE.md` com as camadas e as pegadinhas do `apps/api/CLAUDE.md`, sem nome do
  projeto, e um `.env.example` com cada variável comentada.
- Nas duas, `npm ci`, `npm run lint`, `npm run format:check`, `npm run typecheck`, `npm run build`,
  `npm run test` e `npm run test:e2e` saem com código 0.

### 8. Persistência e Auditoria

- **Modelo de dados:** o mesmo de hoje, com as mesmas entidades, relações, unicidades e enums, criado
  por uma única migration sqlx em `migrations/` no idioma do Postgres: tabelas no plural, colunas em
  snake_case, `text` e `timestamptz`. Nenhum dado é migrado.
- **Auditoria:** a mesma da regra 12 da 003, sem mudança.
- **Eventos/integrações disparados:** os mesmos de hoje, que são o e-mail pelo SMTP do Gmail e o
  OAuth do Google.

---

## Erros

O catálogo não muda e não ganha código. Mensagem é o `title` do Problem Details.

| Código | HTTP | Quando | Mensagem |
| --- | --- | --- | --- |
| `VALIDATION_FAILED` | `400` | 003, regra 10 | "Validation failed" |
| `INVALID_TOKEN` | `400` | 003, tabela Erros | "Invalid token" |
| `TOKEN_EXPIRED` | `400` | 003, tabela Erros | "Token expired" |
| `INVALID_PASSWORD` | `400` | 003, tabela Erros | "Invalid password" |
| `INVALID_CLIENT` | `400` | 004, tabela Erros | "Invalid client" |
| `INVALID_CREDENTIALS` | `401` | 003, tabela Erros | "Invalid email or password" |
| `INVALID_SESSION` | `401` | 003, tabela Erros | "Invalid session" |
| `SESSION_REUSED` | `401` | 003, tabela Erros | "Refresh token reuse detected" |
| `EMAIL_NOT_VERIFIED` | `403` | 003, tabela Erros | "Email not verified" |
| `INVALID_ORIGIN` | `403` | 003, tabela Erros | "Invalid origin" |
| `RATE_LIMITED` | `429` | 003, regra 11 | "Too many requests" |
| `SERVICE_UNAVAILABLE` | `503` | 003, regras 2 e 11 | "Service temporarily unavailable" |

`INVALID_STATE`, `UNVERIFIED_PROVIDER_EMAIL` e `PROVIDER_ERROR` continuam saindo só em `?error=` do
`302` de `/oauth/google/callback`.

## Efeitos Colaterais

- **Persistência:** a mesma das specs de origem.
- **Concorrência:** a mesma. O teste de corrida da rotação do refresh e do registro de envio abre as
  conexões antes de disparar as requisições, como hoje faz o `openConnections()`.
- **Transação:** a mesma das specs de origem, aberta só no `queries.rs`.

---

## Cenários de Aceite (Gherkin)

### Cenário 1 — Os cenários das specs de origem passam contra o Rust (caminho feliz, regras 1 e 8)

```gherkin
Dado o workspace em apps/api com Postgres e Redis de pé
Quando roda `cargo test`
Então cada cenário das specs 002, 003 e 004 cuja rota existe hoje tem um teste em crates/app/tests/
E todos saem verdes contra o schema criado só pela migration de migrations/
```

### Cenário 2 — O cliente Dart muda só nos renomes decididos (caminho feliz, regra 2)

```gherkin
Dado a API Rust de pé com APP_ENV=development
Quando o cliente do apps/mobile é regenerado a partir de `GET /api-json`
Então o diff de `apps/mobile/lib/shared/api` tem só os renomes das ADRs 0010 e 0011
E `flutter analyze --fatal-infos` e `flutter test` do apps/mobile saem 0
```

### Cenário 3 — Variável ausente ou inválida derruba o boot (exceção, regra 3)

```gherkin
Dado o ambiente completo sem JWT_SECRET e com PORT=abc
Quando o binário sobe
Então o stderr contém "Invalid environment:"
E a linha "  JWT_SECRET: expected a string with at least 32 characters"
E a linha "  PORT: expected an integer between 1 and 65535", nessa ordem
E o processo sai com código 1 sem abrir a porta
```

### Cenário 4 — O health check não é logado (caminho alternativo, regra 4)

```gherkin
Dado a API de pé com LOG_LEVEL=info
Quando recebe `GET /health` e depois `GET /rota-inexistente`
Então o stdout tem uma linha só, com método GET, path /rota-inexistente, status 404 e duração
```

### Cenário 5 — Nenhum segredo sai no log (exceção, regra 4)

```gherkin
Dado a API de pé com LOG_LEVEL=trace
Quando recebe `POST /sessions` com senha e depois `GET /sessions/current` com `Authorization: Bearer <token>` e o cookie de sessão
Então nenhuma linha do stdout contém a senha, o token, o valor do cookie nem a DATABASE_URL
```

### Cenário 6 — O OpenAPI não é servido em produção (exceção, regra 3)

```gherkin
Dado a API de pé com APP_ENV=production
Quando recebe `GET /api-json` e `GET /api`
Então responde 404 nas duas, com Problem Details `about:blank`
```

### Cenário 7 — As consultas batem com o schema (exceção, regra 5)

```gherkin
Dado o workspace compilando
Quando uma consulta de um queries.rs não bate com o schema das migrations
Então `cargo build` falha, porque o `query!` confere a consulta na compilação
E o CI roda `cargo sqlx prepare --workspace --check` e falha se o .sqlx/ não bate com as consultas
```

### Cenário 8 — O scaffold compila e não carrega o nome do projeto (caminho feliz, regra 7)

```gherkin
Dado ~/Projects/nestjs-scaffold e ~/Projects/nestjs-scaffold-auth com o Postgres e o Redis do compose de cada um de pé
Quando roda npm ci, lint, format:check, typecheck, build, test e test:e2e em cada pasta
Então todos saem com código 0
E `grep -ri clinicore` fora de node_modules não encontra nada nas duas
```

### Cenário 9 — O scaffold base não tem auth (caminho alternativo, regra 7)

```gherkin
Dado ~/Projects/nestjs-scaffold
Quando se procura src/features/auth
Então a pasta não existe e o AppModule não importa AuthModule
```

### Cenário 10 — A troca preserva o NestJS numa tag (caminho feliz, regra 6)

```gherkin
Dado a task 7 mergeada na develop
Quando se lista apps/api
Então existe Cargo.toml e não existe package.json
E a tag api-nestjs-final aponta para um commit em que apps/api/package.json existe
E o CI tem o job api rodando os gates do Rust e nenhum job Node para a API
```

---

## Fora de Escopo

- **#71, a fila e o worker.** O Rust não tem par oficial do BullMQ, e a fila e o agendador são
  escolhidos na spec dela, que leva a purga para o `crates/worker/`.
- **#94, #100 e #109.** Congeladas até a task 7; são implementadas depois, já em Rust.
- **Dockerfile e deploy.** Não existem hoje para a API.
- **O `compose.yaml` da raiz**, que o `CLAUDE.md` cita e não existe. Resolvido quando o deploy for
  especificado.
- **Git e remote dos scaffolds.** São pastas; o que fazer com elas é decisão posterior do mantenedor.
- **Migração de dado.** Não há produção.
- **Mudança no `apps/web`, no `apps/site` e no `apps/mobile`.** O contrato não muda, então eles não
  mudam.

## Quebra em Tasks

| # | Issue | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- | --- |
| 1 | #115 | Extract the NestJS scaffold into two reusable folders | `~/Projects/nestjs-scaffold` e `~/Projects/nestjs-scaffold-auth`, cópia do `apps/api` com os nomes trocados, `CLAUDE.md` e `.env.example` genéricos | Cenários 8 e 9 | — |
| 2 | #116 | Boot apps/api-rs with validated env, logging, problem details and health | workspace `http`/`app`/`infra`, `rust-toolchain.toml`, `config.rs`, telemetria, `ApiError`, guard de `Origin`, `/health`, OpenAPI fora de produção, `compose.yaml`, job `api-rs` no lugar do `api`, linha do `apps/api-rs` no `CLAUDE.md` raiz | Cenários 3, 4, 6 e 7; cenários 1 e 2 da 002; cenários 28 e 29 da 003 | — |
| 3 | #117 | Sign up and request the verification email on apps/api-rs | migration inicial, `POST /users`, `POST /email-verifications`, e-mail pelo `lettre`, limite por IP no Redis, limite por endereço em `emailDispatch` | Cenários 1, 2 e 32 da 003 | 2 |
| 4 | #118 | Open, read, refresh and close sessions on apps/api-rs | `/sessions`, `/sessions/current`, `/sessions/current/tokens`, `POST /email-verifications/confirmation`, JWT, denylist no Redis, teto de 5, transporte por cookie e por `Clinicore-Client: mobile` | Cenário 5; cenários 3 a 10, 23 a 27, 30 e 31 da 003; cenários 1 a 7 e 16 da 004 | 3 |
| 5 | #119 | Reset and change the password on apps/api | `/password-resets[/confirmation]`, `PUT /users/me/password` | Cenários 11, 12 e 16 a 22 da 003 | 4 |
| 6 | #120 | Sign in with Google on apps/api | `/oauth/google` e `/oauth/google/callback`, vínculo com a conta existente | Cenários 13, 14 e 15 da 003 | 4 |
| 7 | #121 | Replace apps/api with the Rust implementation | tag `api-nestjs-final`, apaga o NestJS, renomeia `apps/api-rs` → `apps/api`, job `api`, reescreve `apps/api/CLAUDE.md` e o `CLAUDE.md` raiz, regenera o cliente Dart | Cenários 1, 2 e 10 | 1, 4 |

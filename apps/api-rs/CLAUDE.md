# Clinicore `apps/api-rs` — camadas e pegadinhas

Aditivo ao `CLAUDE.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este arquivo
apenas onde ela falar do mesmo assunto. A decisão está em
`docs/decisions/0009-api-em-rust-com-axum-e-sqlx.md` e a migração em
`docs/specs/005-migrar-api-para-rust.md`. Na #121 esta pasta vira `apps/api`, e este arquivo vai junto.

## Comandos

Cada comando roda de dentro do `apps/api-rs`. A toolchain é a do `rust-toolchain.toml` (Rust 1.98.1,
edition 2024).

| Gate             | Comando                                                                                                                 |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Formato          | `cargo fmt --check`                                                                                                     |
| Análise estática | `cargo clippy --all-targets -- -D warnings`                                                                             |
| Build            | `cargo build`                                                                                                           |
| Schema           | `infisical run --path=/api -- sqlx migrate run` · `infisical run --path=/api -- cargo sqlx prepare --workspace --check` |
| Testes           | `cargo test`                                                                                                            |

O `sqlx-cli` é instalado com
`cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features postgres,rustls`,
a mesma versão do job `api-rs` do CI. O `compose.yaml` sobe o Postgres 18 e o Redis 8 de
desenvolvimento com `infisical run --path=/api -- docker compose up -d --wait`, e as portas e as credenciais vêm do
Infisical.

## Camadas

**Um crate por camada, e a dependência aponta para baixo: `api-http → api-app → api-infra`.** Quem a
faz valer é o compilador: o `api-http` não declara o `api-infra`, então um handler que chama um
repository não compila.

- **As pastas são `crates/http`, `crates/app` e `crates/infra`; os pacotes têm o prefixo `api-`.** Um
  pacote chamado `http` colidiria com o crate `http` do crates.io, que o axum usa, e o `cargo -p http`
  ficaria ambíguo.
- **`api-http`** é o binário `api`: boot, `Router`, handlers, extractors, DTO, OpenAPI, `ApiError` e
  telemetria. Não tem regra de negócio.
- **`api-app`** tem os services, o domínio e o `AppError`. Ele reexporta o `config` do `api-infra`,
  porque é o único caminho do `api-http` até ele.
- **`api-infra`** tem a configuração, os repositories, o Postgres, o Redis e o e-mail. O `PgPool` não é
  `pub`, então um service não abre consulta.
- **A feature é um módulo dentro de cada crate**, nunca um crate próprio.
- **SQL só nos repositories do `api-infra`, e só por `query!` ou `query_as!`.** SQL montado por
  `format!` ou concatenação é proibido. Transação só no repository. O `.sqlx/` é commitado, e o CI
  roda o `prepare --check`.
- **A URL é um recurso, nunca um verbo** (ADR 0006). Substantivo no plural e em kebab-case, e o método
  HTTP diz a operação. Token nunca vai no path. A única exceção é o OAuth, em `/oauth/<provedor>`.
- **Todo erro é Problem Details da RFC 9457** (ADR 0006), em `application/problem+json`, montado só
  pelo `crates/http/src/error.rs`:
  - O catálogo (`ErrorCode` → status, slug e `title`) mora no `catalog()`, e o `type` é
    `tag:clinicore.com.br,2026:<slug>`.
  - `blank_problem(status)` é o único montador de `about:blank`, usado pelo fallback, pelo panic e
    pelo `AppError::Internal`.
  - `AppError::Internal` e panic viram `500` sem detalhe, com a causa logada em `error`.

## Segredos

- **Os valores de desenvolvimento moram no Infisical, no plano grátis, no ambiente `dev` do projeto `Clinicore`, na pasta `/api`.**
  Nenhum `.env` com valor real fica no disco, e ninguém manda variável para ninguém.
- **O `.infisical.json` é commitado.** Ele liga esta pasta ao projeto e não tem segredo.
- **Todo comando que precisa de variável roda dentro do `infisical run --path=/api --`**: `cargo run`,
  `docker compose up`, `sqlx migrate run` e `cargo sqlx prepare`. O `--env` padrão já é `dev`.
- **Um valor só da sua máquina é override pessoal**, com
  `infisical secrets set <NOME>=<valor> --type=personal`, e o resto do time continua com o valor
  compartilhado. É o caso de uma porta que já está ocupada.
- **O `.env.example` é a lista dos nomes e do formato.** Variável nova entra nele, no `config.rs` e no
  Infisical ao mesmo tempo.
- **O agente não roda comando dentro do `infisical run` sem necessidade.** O `cargo test` não precisa de
  segredo. E nunca roda `infisical secrets`, `infisical export`, `env`, `printenv` nem
  `docker compose config`, porque todos imprimem os valores.

## Configuração

- **As 17 variáveis são lidas uma vez, em `api-infra/src/config.rs`, e nenhuma tem default.** Com uma
  ou mais recusadas, o processo escreve `Invalid environment:` e uma linha por variável, em ordem
  alfabética, e sai com código 1 antes do bind.
- **O binário não lê arquivo `.env`.** As variáveis chegam pelo processo, injetadas pelo
  `infisical run` em dev e pelo orquestrador fora dele.
- **`Config::from_source` recebe a função de leitura.** O teste monta o `Config` a partir de um mapa, e
  nunca do ambiente do processo.

## Log

- **O log sai pelo `tracing` em `stdout`**, em JSON fora de `development` e legível em `development`.
- **Cada requisição gera uma linha só, pelo `TraceLayer`**, com método, path sem query, status e
  `duration_ms`. O layer não lê header, cookie nem corpo. `GET` e `HEAD /health` recebem
  `Span::none()` e não geram linha.
- **O `LOG_LEVEL` vale para os crates `api-*`.** As dependências ficam em `warn` no máximo, para o trace
  do hyper não entrar no log.

## Pegadinhas da stack

- **O `sqlx-cli` procura um `.env` subindo pelos diretórios pais, e o erro de parse dele imprime a
  linha que falhou.** Um valor sem aspas e com espaço basta para pôr o segredo no terminal. É mais um
  motivo para nenhum `.env` com valor real ficar no disco.
- **O `fmt()` do `tracing-subscriber` nasce limitado a `INFO`.** Sem o `.with_max_level(level)`, o
  `Targets` só consegue estreitar, e `debug` e `trace` somem.
- **O guard de origem entra por `route_layer`**, para rota inexistente e método não aceito
  continuarem `404` com qualquer origem. **O `CatchPanicLayer` fica dentro do `CorsLayer`**, para o
  `500` do panic sair com os headers de CORS.
- **Rota nova entra antes do `serve_layers`.** O `method_not_allowed_fallback` só pega as rotas que já
  existem quando ele é chamado, e as camadas só envolvem o que está no `Router` naquele momento. O
  teste que precisa de uma rota própria faz `serve_layers(routes(&config).route(...), &config)`.
- **Método não aceito responde `404 about:blank`, não 405**, porque é o que o Express respondia e o
  contrato não muda.
- **O corpo JSON do contrato é uma struct, nunca `json!`.** O `serde_json` sem `preserve_order` ordena
  as chaves do `json!` em ordem alfabética, e a ordem deixa de ser a do contrato.
- **O `utoipa-swagger-ui` usa a feature `vendored`.** Sem ela, o build baixa a UI da internet.
- **O teste de boot roda o binário com `env_clear()`**, para não enxergar o ambiente de quem roda a
  suíte.

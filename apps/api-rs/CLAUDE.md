# Clinicore `apps/api-rs` — camadas e pegadinhas

Aditivo ao `CLAUDE.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este arquivo
apenas onde ela falar do mesmo assunto. A stack está em
`docs/decisions/0009-api-em-rust-com-axum-e-sqlx.md`, a organização dos crates em
`docs/decisions/0010-api-rs-com-crate-por-processo.md` e a migração em
`docs/specs/005-migrar-api-para-rust.md`. Na #121 esta pasta vira `apps/api`, e este arquivo vai junto.

**O contrato é o HTTP que o web e o mobile consomem; a forma é a do Rust.** URL, status, cookies,
limites, e-mail enviado e o formato Problem Details são contrato. Códigos de campo, validação, schema,
transação e nomes internos seguem o idioma do Rust, do axum e do Postgres, e o `apps/api` não é
referência de implementação.

## Comandos

Cada comando roda de dentro do `apps/api-rs`. A toolchain é a do `rust-toolchain.toml` (Rust 1.98.1,
edition 2024).

| Gate             | Comando                                                                                                                 |
| ---------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Formato          | `cargo fmt --check`                                                                                                     |
| Análise estática | `cargo clippy --all-targets -- -D warnings`                                                                             |
| Build            | `cargo build`                                                                                                           |
| Schema           | `infisical run --path=/api -- sqlx migrate run` · `infisical run --path=/api -- cargo sqlx prepare --workspace --check` |
| Testes           | `infisical run --path=/api -- cargo test`                                                                               |

O `sqlx-cli` é instalado com
`cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features postgres,rustls`,
a mesma versão do job `api-rs` do CI. O `compose.yaml` sobe o Postgres 18 e o Redis 8 de
desenvolvimento com `infisical run --path=/api -- docker compose up -d --wait`, e as portas e as credenciais vêm do
Infisical.

## Crates

**Um crate por processo ou capacidade, em `crates/`** (ADR 0010). O pacote tem o
prefixo `clinicore-`, porque `core` é o nome da biblioteca base do Rust.

- **`crates/core/` (`clinicore-core`)** tem a configuração, o pool do Postgres, o Redis e o e-mail. Serve a
  qualquer processo e não conhece feature.
- **`crates/app/` (`clinicore-app`)** é a API inteira e gera o binário `api`.
- **`worker/` e `transfer/` só nascem com código**: o worker no primeiro job da fila, o transfer na
  importação e exportação de arquivos.
- **Dependência nova entra em `[workspace.dependencies]` do `Cargo.toml` da raiz**, com a versão
  exata e as features comuns; o crate declara `nome.workspace = true` e só acrescenta a feature que é
  dele.

## Estrutura

```
crates/
├── core/
│   ├── templates/  mail.html  mail.txt              layout do e-mail, em askama
│   └── src/  lib.rs  config.rs  db.rs  redis.rs  mail.rs
└── app/
    ├── src/
    │   ├── main.rs                                  liga o servidor
    │   ├── lib.rs                                   AppState, Router e camadas do tower
    │   ├── telemetry.rs                             o subscriber do tracing, do processo inteiro
    │   ├── http/                                    o encanamento HTTP
    │   │   ├── mod.rs
    │   │   ├── error.rs  validation.rs  request_log.rs  openapi.rs
    │   │   └── origin.rs  rate_limit.rs
    │   ├── health.rs                                feature de uma rota só
    │   ├── users/                                   o cadastro
    │   │   └── mod.rs  handlers.rs  requests.rs  service.rs  queries.rs
    │   └── auth/                                    verificação de e-mail e sessão
    │       ├── mod.rs                               routes(): o Router, que liga URL e handler
    │       ├── handlers.rs                          recebe o request, chama o service, devolve o status
    │       ├── requests.rs                          corpos que chegam e a validação deles
    │       ├── service.rs                           a regra
    │       ├── queries.rs                           o SQL
    │       ├── error.rs                             o erro da feature e o From para o AppError
    │       └── password.rs  token.rs  emails.rs     o que a regra usa
    └── tests/api/
        ├── main.rs  support.rs
        ├── boot.rs  errors.rs  health.rs  openapi.rs  request_log.rs
        └── users/  mod.rs  sign_up.rs
migrations/  .sqlx/
```

- **A raiz do `src/` tem o boot, a telemetria, o `http/` e as features.** O que serve a qualquer
  rota (erro, extractor de validação, log de requisição, OpenAPI, middleware) fica em `http/`, sem
  subpasta. Regra de uma feature, como a política de senha, fica na feature.
- **A feature começa como `<feature>.rs`, com o `Router`, os handlers e o SQL juntos.** Ela vira a
  pasta `<feature>/` quando um pedaço tiver responsabilidade própria, e o `<feature>.rs` vira o
  `<feature>/mod.rs`. Arquivo com o nome de uma pasta ao lado dela não é usado.
- **Na pasta, o fluxo se lê pelos arquivos: `mod.rs` → `handlers.rs` → `service.rs` → `queries.rs`.**
  O `mod.rs` só declara os módulos e monta o `routes()`. O nome é o do axum: handler, não controller.
- **A camada nasce quando tem conteúdo.** Uma leitura simples vai do handler direto à consulta. O
  `service.rs` existe quando há decisão: regra de negócio, mais de uma escrita ou efeito colateral.
  Nenhum service só repassa a chamada.
- **O estado da API é o `AppState`**, com o pool, o Redis, o `Mailer` e o que a configuração entrega
  às features. O handler recebe `State<AppState>`, e o service e a consulta recebem o que usam.
- **O `core` monta o e-mail e a feature escreve o texto.** O layout é o `askama`, em
  `crates/core/templates/`, e o `mail::compose` o renderiza; o conteúdo de cada e-mail mora em
  `<feature>/emails.rs`.

## Regras

- **A ordem de uma rota limitada é origem, limite, validação.** O guard de `Origin` é `route_layer` do
  `with_layers`, o limite é `route_layer` da própria rota e a validação é o extractor `ValidJson`, então
  uma origem recusada não conta no limite e um corpo inválido conta.
- **O corpo é validado por `serde` e `validator`, pelo extractor `ValidJson`.** O `code` de cada
  campo é o do `validator` (`email`, `length`, e o nome do validador próprio, como `weak_password`).
  O request tem `#[serde(deny_unknown_fields)]`, e campo desconhecido ou de tipo errado sai como
  `invalid`. JSON malformado sai como `400 about:blank`, e content-type que não é JSON como `415`.
  O tipo Rust é `<Operação>Request`, e é também o nome do schema no OpenAPI.
- **Concorrência se resolve no Postgres, sem retry na aplicação.** Unicidade por
  `ON CONFLICT … DO NOTHING`, e a serialização por chave (o registro de envio por endereço) por
  `pg_advisory_xact_lock` dentro da transação.
- **SQL só no `queries.rs` da feature, e só por `query!` ou `query_as!`.** SQL montado por `format!`
  ou concatenação é proibido. Transação só no `queries.rs`. O compilador não impede um handler de usar
  o pool, então a regra é conferida no review. O `.sqlx/` é commitado, e o CI roda o
  `prepare --check`.
- **A URL é um recurso, nunca um verbo** (ADR 0006). Substantivo no plural e em kebab-case, e o método
  HTTP diz a operação. Token nunca vai no path. A única exceção é o OAuth, em `/oauth/<provedor>`.
- **Todo erro é Problem Details da RFC 9457** (ADR 0006), em `application/problem+json`, montado só
  pelo `crates/app/src/http/error.rs`:
  - O `AppError` é um `enum` do `thiserror` e implementa o `IntoResponse`. Handler, extractor e
    middleware devolvem `Result<_, AppError>`, e ninguém fora do `error.rs` monta resposta de erro.
  - Cada variante com código leva status, slug e `title`; o `type` é
    `tag:clinicore.com.br,2026:<slug>`. `Rejected(status)` é o `about:blank` do protocolo: 404, 415,
    JSON malformado e corpo grande.
  - O `core` é biblioteca e não conhece HTTP: cada módulo devolve o erro tipado dele (`MailError`,
    `RedisError`, `sqlx::Error`, `InvalidEnvironment`), nunca `Box<dyn Error>`.
  - A feature tem o próprio `enum` em `<feature>/error.rs` e o `impl From<ErroDaFeature> for
    AppError`. Service e helpers da feature devolvem o erro dela, e o handler converte com `?`.
  - O `sqlx::Error` vira `AppError::Database` e o `RedisError` vira `AppError::Unavailable` pelo
    `#[from]`, para o handler que chama o `core` direto.
  - `Database`, `Internal` e panic viram `500` sem detalhe, e `Unavailable` vira `503`; a causa é
    logada em `error` só no `error.rs`.

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
- **O agente não roda comando dentro do `infisical run` sem necessidade.** E nunca roda `infisical secrets`, `infisical export`, `env`, `printenv` nem
  `docker compose config`, porque todos imprimem os valores.

## Configuração

- **As 17 variáveis são lidas uma vez, em `crates/core/src/config.rs`, e nenhuma tem default.** Com uma
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
- **O `LOG_LEVEL` vale para os crates `clinicore-*`.** As dependências ficam em `warn` no máximo, para o trace
  do hyper não entrar no log.

## Testes

- **O `cargo test` precisa do Postgres e do Redis de pé e roda dentro do `infisical run`.** O
  `#[sqlx::test(migrations = "../../migrations")]` cria um banco por teste a partir da `DATABASE_URL`.
- **O Redis dos testes é o de dev, sem flush.** Cada requisição de teste sai de um `fresh_client()`, um
  /64 de documentação novo por chamada, então nenhum contador de limite atravessa testes nem execuções.
- **A consulta do teste é `sqlx::query_scalar` em runtime, com SQL literal.** O `.sqlx/` cobre só as
  consultas dos `queries.rs`.
- **Teste unitário de lógica pura fica no próprio arquivo**, num `#[cfg(test)] mod tests` no fim, como
  o do `crates/core/src/config.rs`.
- **O comportamento que o cliente observa é teste de integração em `crates/app/tests/api/`.** É um binário
  só, `tests/api/main.rs`, com um módulo por feature e o `support.rs` com os helpers. Arquivo solto em
  `tests/` vira outro binário e é proibido.

## Pegadinhas da stack

- **O `sqlx-cli` procura um `.env` subindo pelos diretórios pais, e o erro de parse dele imprime a
  linha que falhou.** Um valor sem aspas e com espaço basta para pôr o segredo no terminal. É mais um
  motivo para nenhum `.env` com valor real ficar no disco.
- **O `fmt()` do `tracing-subscriber` nasce limitado a `INFO`.** Sem o `.with_max_level(level)`, o
  `Targets` só consegue estreitar, e `debug` e `trace` somem.
- **O guard de origem entra por `route_layer`**, para rota inexistente e método não aceito
  continuarem `404` com qualquer origem. **O `CatchPanicLayer` fica dentro do `CorsLayer`**, para o
  `500` do panic sair com os headers de CORS.
- **Rota nova entra antes do `with_layers`.** O `method_not_allowed_fallback` só pega as rotas que já
  existem quando ele é chamado, e as camadas só envolvem o que está no `Router` naquele momento. O
  teste que precisa de uma rota própria faz `with_layers(routes(&config, lazy_state(&config)).route(...), &config)`.
- **Método não aceito responde `404 about:blank`, não 405**, porque é o que o Express respondia e o
  contrato não muda.
- **O corpo JSON do contrato é uma struct, nunca `json!`.** O `serde_json` sem `preserve_order` ordena
  as chaves do `json!` em ordem alfabética, e a ordem deixa de ser a do contrato.
- **O `utoipa-swagger-ui` usa a feature `vendored`.** Sem ela, o build baixa a UI da internet.
- **O IP do cliente vem do `axum-client-ip`, pela fonte em `CLIENT_IP_SOURCE`.** Sem proxy na frente é
  `ConnectInfo`, o socket, que não se forja. Atrás de proxy é o header que só ele escreve
  (`RightmostXForwardedFor`, `CfConnectingIp`, …), e a API só pode ser alcançável por ele: exposta
  direto, o header vem do cliente.
- **O bind em `::` entrega o IPv4 como `::ffff:a.b.c.d`.** O `client_ip` passa o endereço por
  `to_canonical()` antes de montar a chave do limite, e agrupa o IPv6 em /64.
- **O `MockConnectInfo` do axum não põe `ConnectInfo` nas extensions**, só o extractor o enxerga. O
  teste insere `ConnectInfo` direto na requisição, porque o `client_ip` lê as extensions.
- **Postgres e Redis conectam sob demanda.** O boot não espera nenhum dos dois, e com o Redis fora a
  rota limitada responde `503` enquanto o `/health` continua `200`.
- **O teste de boot roda o binário com `env_clear()`**, para não enxergar o ambiente de quem roda a
  suíte.

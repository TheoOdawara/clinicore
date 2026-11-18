# `apps/api` — forma e regras

Aditivo ao `AGENTS.md` da raiz. Stack nas ADRs 0009 e 0019, crates na 0010, OpenAPI na 0011, revogação
na 0012, forma na 0026. O contrato é o HTTP (URL, status, cookie, limite, Problem Details); o resto
segue o idioma do Rust, do axum e do Postgres.

## Crates

- **`crates/core` (`clinicore-core`)**: configuração, Postgres, Redis, layout do e-mail, sinal de parada,
  `tracing` e os prazos que a API e o worker dividem. Não conhece feature nem HTTP.
- **`crates/app` (`clinicore-app`)**: a API inteira; gera o binário `api`.
- **`crates/worker` (`clinicore-worker`)**, que nasce com o primeiro job: o trabalho fora da requisição
  (fila de e-mail, purga às 03:00 de São Paulo); gera o binário `worker` e depende só do `core`.
- Dependência nova entra em `[workspace.dependencies]` com a versão exata.

## Estrutura

```
crates/app/src/
├── main.rs  lib.rs          boot; AppState, Router e camadas do tower
├── http/                    erro, ValidJson, log, OpenAPI, cliente, origem, limite
├── credentials/             senha e segredo, usados por várias features
├── tenancy/ authorization/ audit/ idempotency/    transversais, nascem com a spec de cada um
├── <feature>.rs             a feature começa num arquivo, com rota, handler e SQL
└── <feature>/               e vira pasta quando um pedaço tem responsabilidade própria
    ├── mod.rs               declara os módulos e monta o routes()
    ├── handlers.rs          recebe, chama o service, devolve o status
    ├── requests.rs  responses.rs
    ├── service.rs           só quando há regra, mais de uma escrita ou efeito colateral
    ├── queries.rs           o SQL e as transações
    ├── error.rs             o enum da feature e o From para o AppError
    └── <grupo>/             subpasta para o que tem consumidor ou ciclo próprio (sessions/tokens/)
crates/app/tests/api/<feature>/<operação>.rs    um binário só: main.rs e support.rs
crates/worker/src/<job>.rs · crates/worker/tests/worker/
migrations/ · .sqlx/ (commitado)
```

O módulo tem o nome do recurso no plural, igual à URL (`password_resets` ↔ `/password-resets`).

## Fronteiras

- **Uma feature usa de outra só `service`, `error`, `handlers` e `extractors`.** O `queries` é sempre
  `mod queries;`, privado.
- **SQL só no `queries.rs`**, ou no próprio arquivo enquanto a feature for um arquivo só. Sempre por
  `query!`/`query_as!`; SQL montado por `format!` é proibido.
- **Resposta de erro só no `http/error.rs`.** Todo erro é Problem Details (ADR 0006); handler, extractor
  e middleware devolvem `Result<_, AppError>`.
- **Rota só por `routes!` do `utoipa-axum`**, com `#[utoipa::path]` (ADR 0011).
- **O handler recebe `State<AppState>`; o service recebe `&AppState`; a consulta recebe o pool ou o
  Redis.**
- **`Result` para falha do ambiente; `expect` com a invariante na mensagem** para o que só falha com o
  código errado. Valor de fora vira tipo na fronteira: ID com tipo próprio (`UserId`) e `Email`
  normalizado.
- **O `tests/api/architecture.rs` reprova** SQL fora do lugar, `IntoResponse` fora do `http/error.rs` e
  `.route(`.

## Nomes

`<Operação>Request` e `<Recurso>Response`, que são também o nome do schema no OpenAPI; `<Feature>Error`;
`<Entidade>Id`.

## Configuração

O ambiente é lido só em `crates/core/src/config.rs`, sem default, e o boot falha listando cada variável
recusada. O binário não lê `.env`; tudo que precisa de variável roda sob `infisical run --path=/api --`.

## Caminhos críticos

100% das linhas novas (ADR 0023): `sessions/`, `credentials/`, `oauth/`, `password_resets/`,
`email_verifications/`, `http/rate_limit.rs`, `http/origin.rs`, `http/client.rs`, `tenancy/`,
`authorization/`, `audit/`, `idempotency/`, e no worker `purge.rs` e `outbox.rs`.

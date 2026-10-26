# Clinicore `apps/api` — camadas e pegadinhas

Aditivo ao `AGENTS.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este arquivo
apenas onde ela falar do mesmo assunto. A stack está em
`docs/decisions/0009-api-em-rust-com-axum-e-sqlx.md`, a organização dos crates em
`docs/decisions/0010-api-rs-com-crate-por-processo.md`, o documento OpenAPI em
`docs/decisions/0011-openapi-da-api-rs-nasce-com-a-rota.md`, a ordem da revogação em
`docs/decisions/0012-revogacao-de-sessao-antes-de-apagar.md` e a migração em
`docs/specs/005-migrar-api-para-rust.md`.

**O contrato é o HTTP que o web e o mobile consomem; a forma é a do Rust.** URL, status, cookies,
limites, e-mail enviado e o formato Problem Details são contrato. Códigos de campo, validação, schema,
transação e nomes internos seguem o idioma do Rust, do axum e do Postgres, e o NestJS que a API foi,
guardado na tag `api-nestjs-final`, não é referência de implementação.

## Comandos

Cada comando roda de dentro do `apps/api`. A toolchain é a do `rust-toolchain.toml` (Rust 1.99.1,
edition 2024).

| Gate | Comando |
| --- | --- |
| Formato | `cargo fmt --check` |
| Análise estática | `cargo clippy --all-targets -- -D warnings` |
| Build | `cargo build` |
| Schema | `infisical run --path=/api -- sqlx migrate run` · `infisical run --path=/api -- cargo sqlx prepare --workspace --check` |
| Dependências | `cargo deny check` |
| Testes e cobertura | `infisical run --path=/api -- cargo llvm-cov nextest --fail-under-lines 100 --fail-under-functions 100 --fail-under-regions 100` |

O `sqlx-cli` é instalado com
`cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features postgres,rustls`, e
o `cargo-nextest`, o `cargo-llvm-cov` e o `cargo-deny` com `cargo install --locked`, nas versões do job
`api` do CI. O `llvm-tools-preview` que o `cargo-llvm-cov` usa vem do `rust-toolchain.toml`.

**As advisories ignoradas ficam no `deny.toml`**, cada uma com o motivo no campo `reason`:

- **RUSTSEC-2023-0071**: o ataque de tempo do `rsa` alcança operação com chave privada RSA, e a API não
  tem nenhuma. O `rsa` vem do `jsonwebtoken`, que só assina e confere HS256, e do `openidconnect`, que só
  confere a assinatura RS256 do `id_token` com a chave pública do Google.
- **RUSTSEC-2026-0387**: o `crypto-bigint` 0.5 vem pelo `p256` 0.13, que o `jsonwebtoken` 11.1.0 e o
  `openidconnect` 4.0.1 trazem, os dois na última versão. Nenhum caminho usa ES256. O ignore sai quando
  os dois subirem o `p256`.
- **RUSTSEC-2026-0374**: o `basic-toml` está sem manutenção, mas só o `askama_derive` o usa, na
  compilação, para ler o `askama.toml`. Nada chega ao binário.

**A cobertura é 100% em linhas, funções e regiões.** O Rust stable não mede branch: o `--branch` do
`llvm-cov` exige nightly, e região é o mais perto disso. O nextest não roda doctest, e o repo não tem
nenhum, porque o contrato só aceita comentário `shortcut:`.

O `compose.yaml` sobe o Postgres 18.6 e o Redis 8.10.2 de
desenvolvimento com `infisical run --path=/api -- docker compose up -d --wait`, e as portas e as credenciais vêm do
Infisical.

## Crates

**Um crate por processo ou capacidade, em `crates/`** (ADR 0010). O pacote tem o
prefixo `clinicore-`, porque `core` é o nome da biblioteca base do Rust.

- **`crates/core/` (`clinicore-core`)** tem a configuração, o pool do Postgres, o Redis, o template do
  e-mail, o sinal de parada, o subscriber do `tracing` e os prazos que mais de um processo usa. Serve a qualquer processo e não
  conhece feature.
- **`crates/app/` (`clinicore-app`)** é a API inteira e gera o binário `api`.
- **`crates/worker/` (`clinicore-worker`)** gera o binário `worker`, que faz o trabalho fora da
  requisição, e depende só do `clinicore-core`.
- **`transfer/` só nasce com código**, na importação e exportação de arquivos.
- **Dependência nova entra em `[workspace.dependencies]` do `Cargo.toml` da raiz**, com a versão
  exata e as features comuns; o crate declara `nome.workspace = true` e só acrescenta a feature que é
  dele.

## Estrutura

```
crates/
├── core/
│   ├── templates/  mail.html  mail.txt              layout do e-mail, em askama
│   └── src/  lib.rs  config.rs  db.rs  redis.rs  mail.rs  shutdown.rs  telemetry.rs  lifetimes.rs
├── worker/
│   ├── src/
│   │   ├── main.rs                              liga o worker e espera o sinal de parada
│   │   ├── outbox.rs                            reserva, envia e tenta de novo o e-mail da fila
│   │   ├── mail.rs                              o Mailer, o SMTP do lettre
│   │   ├── schedule.rs                          o cron da purga, às 03:00 de São Paulo
│   │   └── purge.rs                             apaga o que venceu
│   └── tests/worker/  main.rs  support.rs  boot.rs  outbox.rs  purge.rs  schedule.rs
└── app/
    ├── src/
    │   ├── main.rs                                  liga o servidor
    │   ├── lib.rs                                   AppState, Router e camadas do tower
    │   ├── http/                                    o encanamento HTTP
    │   │   ├── mod.rs
    │   │   ├── error.rs  validation.rs  request_log.rs  openapi.rs
    │   │   └── client.rs  origin.rs  rate_limit.rs
    │   ├── health.rs                                feature de uma rota só
    │   ├── email_dispatches.rs                      a reserva de envio por endereço, de várias features
    │   ├── email_outbox.rs                          o token e o e-mail na fila, de várias features
    │   ├── credentials/                             senha e segredo aleatório, de várias features
    │   │   └── mod.rs  password.rs  secret.rs  error.rs
    │   ├── users/                                   o cadastro e a troca de senha
    │   │   └── mod.rs  handlers.rs  requests.rs  service.rs  queries.rs  error.rs
    │   ├── email_verifications/                     o link de confirmação do e-mail
    │   │   └── mod.rs  handlers.rs  requests.rs  service.rs  queries.rs  emails.rs  error.rs
    │   ├── password_resets/                         o link de redefinição de senha
    │   │   └── mod.rs  handlers.rs  requests.rs  service.rs  queries.rs  emails.rs  error.rs
    │   ├── oauth/                                   o login com Google e o vínculo com a conta
    │   │   └── mod.rs  handlers.rs  requests.rs  service.rs  queries.rs  google.rs  cookies.rs  error.rs
    │   └── sessions/                                login, refresh e logout
    │       ├── mod.rs                               routes(): o Router, que liga URL e handler
    │       ├── handlers.rs                          recebe o request, chama o service, devolve o status
    │       ├── requests.rs  responses.rs            corpos que chegam, com a validação, e os que saem
    │       ├── service.rs                           a regra
    │       ├── queries.rs                           o SQL
    │       ├── error.rs                             o erro da feature e o From para o AppError
    │       ├── extractors.rs                        CurrentSession e Device
    │       └── tokens/                              o que a sessão entrega ao cliente
    │           └── mod.rs  access.rs  refresh.rs  cookies.rs
    └── tests/api/
        ├── main.rs  support.rs  faults.rs
        ├── boot.rs  cors.rs  errors.rs  health.rs  openapi.rs  request_log.rs
        ├── users/  mod.rs  sign_up.rs  change_password.rs
        ├── email_verifications/  mod.rs  request.rs  confirmation.rs
        ├── oauth/  mod.rs  google_sign_in.rs
        ├── password_resets/  mod.rs  request.rs  confirmation.rs
        └── sessions/  mod.rs  sign_in.rs  current_session.rs  refresh.rs
migrations/  .sqlx/
```

- **A raiz do `src/` tem o boot, a telemetria, o `http/` e as features.** O que serve a qualquer
  rota (erro, extractor de validação, log de requisição, OpenAPI, middleware) fica em `http/`, sem
  subpasta. Regra de uma feature, como a política de senha, fica na feature.
- **A feature começa como `<feature>.rs`, com o `Router`, os handlers e o SQL juntos.** Ela vira a
  pasta `<feature>/` quando um pedaço tiver responsabilidade própria, e o `<feature>.rs` vira o
  `<feature>/mod.rs`. Arquivo com o nome de uma pasta ao lado dela não é usado.
- **Dentro da pasta, um grupo com consumidor ou ciclo de vida próprio vira subpasta**, como os tokens
  em `sessions/tokens/`, e o fluxo da requisição fica plano. Arquivo novo numa feature reabre a
  pergunta para a pasta inteira. Regra que várias features usam e que não é de nenhuma mora numa
  pasta própria em `src/`, como `credentials/`.
- **Na pasta, o fluxo se lê pelos arquivos: `mod.rs` → `handlers.rs` → `service.rs` → `queries.rs`.**
  O `mod.rs` só declara os módulos e monta o `routes()`. O nome é o do axum: handler, não controller.
- **A camada nasce quando tem conteúdo.** Uma leitura simples vai do handler direto à consulta. O
  `service.rs` existe quando há decisão: regra de negócio, mais de uma escrita ou efeito colateral.
  Nenhum service só repassa a chamada.
- **O estado da API é o `AppState`**, com o pool, o Redis, o `GoogleClient` e o que a configuração entrega
  às features. O handler recebe `State<AppState>`, e o service e a consulta recebem o que usam.
- **O `core` monta o e-mail e a feature escreve o texto.** O layout é o `askama`, em
  `crates/core/templates/`, e o `mail::compose` o renderiza; o conteúdo de cada e-mail mora em
  `<feature>/emails.rs`.

## Regras

- **A ordem de uma rota limitada é cliente, origem, limite, validação.** O guard do `Clinicore-Client` e o
  de `Origin` são `route_layer` do `with_layers`, o limite é `route_layer` da própria rota e a validação é
  o extractor `ValidJson`, então uma origem recusada não conta no limite e um corpo inválido conta.
- **Toda rota limitada conta em duas chaves, nunca numa só.** O IP é a do `route_layer`; a segunda é
  a identidade, contada por `rate_limit::enforce` com uma `Quota` do `mod.rs` da feature: a sessão
  por rota no `CurrentSession` (sessão, método e `MatchedPath`), a sessão do refresh e as falhas de
  senha por e-mail no service, as falhas da senha atual por usuário na troca de senha, e a faixa de
  rede nas duas confirmações (`network_key`: /48 no IPv6, /24 no IPv4), porque o corpo delas só traz
  o token, e nas duas rotas do Google, que são navegação pública sem identidade. As rotas que mandam e-mail têm a segunda no `email_dispatches`. Rota nova limitada nasce com as duas.
- **Rota nasce com `routes!` do `utoipa-axum` num `OpenApiRouter`** (ADR 0011), nunca com o `route`
  do axum, então a rota não existe sem o `#[utoipa::path]`. O header `Clinicore-Client` e as respostas
  dos guards (400, 403, 429, 503) entram sozinhos pelo `document_guards`; o handler declara só as
  respostas próprias, com `body = Problem` e `content_type = "application/problem+json"` no erro.
- **O `Clinicore-Client` escolhe o transporte em toda rota.** Ausente é `web`, `mobile` é o token no
  `Authorization`, e qualquer outro valor é `400 invalid-client` antes de tudo. O `SessionClient` é o
  extractor que lê o header, o guard do `with_layers` o extrai em toda rota, e o `CurrentSession` lê o
  token só do transporte declarado.
- **Toda revogação escreve `auth:revoked:<sessão>` no Redis antes de a sessão sumir do Postgres**
  (ADR 0012). O logout e o reuso do refresh revogam e depois apagam, em autocommit; o teto de 5 apaga
  com `RETURNING`, revoga e faz o commit. Com o Redis fora, nada é apagado e a rota responde `503`.
  A chave vive 7 dias, o maior tempo de vida do refresh, e o refresh a consulta antes de rotacionar.
- **A denylist exige um Redis que persiste.** O `compose.yaml` sobe o Redis com `--appendonly yes`, e
  todo ambiente faz o mesmo: um Redis que reinicia vazio devolve a validade aos access tokens de
  sessões apagadas nos últimos 15 minutos e ao refresh de sessões revogadas cuja linha sobreviveu.
- **Apagar um usuário revoga as sessões dele antes do `DELETE`.** O `ON DELETE CASCADE` apaga as
  linhas de `sessions` sem passar pela denylist, e o access token de cada uma continuaria aceito.
- **A purga do que venceu roda no worker, às 03:00 de `America/Sao_Paulo`**, e nunca no boot. Ela apaga
  a sessão vencida ou que passou do teto de 30 dias (`SESSION_ABSOLUTE_LIFETIME`), a verificação vencida
  o envio com mais de 24 horas (`DISPATCH_WINDOW`, a mesma janela do limite de envio) e o e-mail vencido
  da fila. Não escreve na
  denylist, porque o access token de uma sessão vencida já venceu. Os dois prazos ficam no
  `crates/core/src/lifetimes.rs`, porque a API e o worker leem os mesmos. Duas instâncias purgam duas
  vezes, e a segunda apaga zero linhas; uma purga que falha só tenta de novo na 03:00 seguinte.
- **A API nunca fala com o SMTP.** Rota que manda e-mail grava o token e a mensagem pelo
  `email_outbox::enqueue_with_token`, dentro da transação que reserva o envio, e o worker entrega. Depois
  do commit, o e-mail fica na fila até o worker entregá-lo ou desistir dele.
- **O `pointer` do erro de campo é o nome do JSON, em camelCase**, e o `ValidJson` o deriva do nome do
  campo Rust. Request com mais de uma palavra num campo leva `#[serde(rename_all = "camelCase")]`.
- **O corpo é validado por `serde` e `validator`, pelo extractor `ValidJson`.** O `code` de cada
  campo é o do `validator` (`email`, `length`, e o nome do validador próprio, como `weak_password`).
  O request tem `#[serde(deny_unknown_fields)]`, e campo desconhecido ou de tipo errado sai como
  `invalid`. JSON malformado sai como `400 about:blank`, e content-type que não é JSON como `415`.
  O tipo Rust é `<Operação>Request`, e é também o nome do schema no OpenAPI. O `#[serde(default)]`
  faz o utoipa marcar o campo como opcional, então todo campo obrigatório leva também
  `#[schema(required = true)]`; sem ele o cliente Dart o gera como `String?`.
- **Um formato de token tem uma `Regex` só**, que valida o corpo, entra no schema e confere o token
  lido do cookie. O segredo é o `credentials::secret::PATTERN`, e o `secret::FORMAT` é a `Regex` dele
  nos corpos das duas confirmações; o `pattern` do schema sai da mesma
  `Regex` por `schema_with` e `openapi::matching`, porque o `pattern` do utoipa só aceita literal.
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
  - O `core` é biblioteca e não conhece HTTP: cada módulo devolve o erro tipado dele (`RedisError`,
    `sqlx::Error`, `InvalidEnvironment`), nunca `Box<dyn Error>`.
  - A feature tem o próprio `enum` em `<feature>/error.rs` e o `impl From<ErroDaFeature> for
    AppError`. Service e helpers da feature devolvem o erro dela, e o handler converte com `?`.
  - O `sqlx::Error` vira `AppError::Database` e o `RedisError` vira `AppError::Unavailable` pelo
    `#[from]`, para o handler que chama o `core` direto.
  - `Database`, `Internal` e panic viram `500` sem detalhe, e `Unavailable` vira `503`; a causa é
    logada em `error` só no `error.rs`.
- **`Result` é para falha que o ambiente causa; `expect` é para invariante do próprio programa.** Rede,
  banco, Redis, SMTP e dado que vem de fora (um endereço lido do banco, um corpo, um token) voltam por
  `?`. Uma operação que só falha se o código estiver errado usa `expect` com a invariante na mensagem:
  assinar HS256, renderizar um template que compila, sortear do `getrandom`, o semáforo do hash que
  ninguém fecha, o `spawn_blocking` sem cancelamento, o registro do SIGTERM. Valor que pode vir
  malformado vira tipo na fronteira (`PgConnectOptions`, `ConnectionInfo`, `lettre::Address` no
  `config.rs`), e daí para dentro a conversão não falha mais. Variante de erro que nenhum caminho
  produz não existe.

## Segredos

- **Os valores de desenvolvimento moram no Infisical, no plano grátis, no ambiente `dev` do projeto `Clinicore` da org `ATNexusLab`, na pasta `/api`.**
  Ninguém manda variável para ninguém.
- **O `apps/api/.env` é a cópia local desses valores, fora do git.** O binário não o lê; o
  `docker compose` e o `sqlx-cli` o leem sozinhos quando o comando roda sem o `infisical run`.
- **O `.infisical.yaml` é commitado.** Ele liga esta pasta ao projeto e não tem segredo. O
  `.infisical.json` é o formato legado, e o `infisical migrate-config` converte um para o outro.
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

- **As variáveis são lidas uma vez, em `crates/core/src/config.rs`, e nenhuma tem default**: 12 no
  `Config` da API e 8 no `WorkerConfig`, que são as únicas a ler as do SMTP. Com uma
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

- **O `cargo nextest` precisa do Postgres e do Redis de pé e roda dentro do `infisical run`.** O
  `#[sqlx::test(migrations = "../../migrations")]` cria um banco por teste a partir da `DATABASE_URL`.
- **A suíte só lê `DATABASE_URL` e `REDIS_URL` do ambiente**; o resto vem do mapa do `support.rs`. Com
  as portas de dev ocupadas por outro projeto, o gate roda contra um `postgres:18` e um `redis:8`
  descartáveis em portas livres, com as duas variáveis na linha de comando e sem o `infisical run`. O
  binário rodado assim precisa das 12 da API, e o do worker das 8 dele, em que o `SMTP_HOST` só aceita
  hostname com domínio.
- **O Redis dos testes é o de dev, sem flush.** Cada requisição de teste sai de um `fresh_client()`, um
  /64 de documentação novo por chamada, então nenhum contador de limite atravessa testes nem execuções.
  **Pelo mesmo motivo, teste que faz login usa um `fresh_email()`**: o contador de falhas por e-mail
  atravessaria testes e execuções. O teste do teto por faixa de rede usa endereços IPv4 de
  documentação, que o `fresh_client()` nunca gera.
- **A consulta do teste é `sqlx::query_scalar` em runtime, com SQL literal.** O `.sqlx/` cobre só as
  consultas dos `queries.rs`.
- **Teste unitário de lógica pura fica no próprio arquivo**, num `#[cfg(test)] mod tests` no fim, como
  o do `crates/core/src/config.rs`.
- **A falha do Postgres e do Redis no meio de uma requisição é provada pelo `tests/api/faults.rs`.** O
  `faulty_app` põe um proxy TCP entre o app e cada um, e `cut_at(comando, pular)` derruba a conexão
  quando o cliente manda o próximo trecho com aquele texto: um SQL (`"INSERT INTO sessions"`,
  `"COMMIT"`) ou um comando do Redis (`"EVALSHA"`, `"SETEX"`). O pool do proxy usa
  `statement_cache_capacity(0)`, para todo `query!` mandar o SQL de novo, e `PgSslMode::Disable`, para o
  texto passar legível. O worker não sobe proxy: renomeia a tabela ou cria um trigger que levanta erro.
- **O teste de boot sobe o binário de verdade e manda o sinal com `kill -s`**, para o SIGTERM e o
  SIGINT saírem com 0. A API está pronta quando responde `404` numa rota que não existe.
- **O comportamento que o cliente observa é teste de integração em `crates/app/tests/api/`.** É um binário
  só, `tests/api/main.rs`, com um módulo por feature e o `support.rs` com os helpers. Arquivo solto em
  `tests/` vira outro binário e é proibido. O worker segue a mesma forma, em `crates/worker/tests/worker/`.

## Pegadinhas da stack

- **O `sqlx-cli` procura um `.env` subindo pelos diretórios pais, e o erro de parse dele imprime a
  linha que falhou.** Um valor sem aspas e com espaço no `apps/api/.env` basta para pôr o segredo no
  terminal.
- **Com o `apps/api/.env` no disco, as macros do sqlx compilam contra o banco, e não contra o `.sqlx/`.**
  Elas leem o `DATABASE_URL` dele, e com o Postgres fora ou sem as migrations aplicadas o `cargo build`
  e o `cargo clippy` caem em erros de tipo nas consultas. O `SQLX_OFFLINE=true` volta a usar o `.sqlx/`.
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
- **O `Secure` do cookie segue o esquema da `API_URL`**, não o `APP_ENV`: homologação com
  `APP_ENV=development` atrás de HTTPS continua mandando cookie `Secure`.
- **O IP do cliente vem do `axum-client-ip`, pela fonte em `CLIENT_IP_SOURCE`.** Sem proxy na frente é
  `ConnectInfo`, o socket, que não se forja. Atrás de proxy é o header que só ele escreve
  (`RightmostXForwardedFor`, `CfConnectingIp`, …), e a API só pode ser alcançável por ele: exposta
  direto, o header vem do cliente.
- **Em produção a requisição passa pela Cloudflare e depois pelo Traefik do Coolify.** A fonte é
  `CfConnectingIp`, e a origem só aceita tráfego da Cloudflare, por firewall com as faixas dela ou por
  Cloudflare Tunnel: quem alcança o Traefik direto forja o `CF-Connecting-IP`. A porta da API nunca é
  publicada fora do Traefik.
- **O worker em dev envia e-mail de verdade.** O `cargo run -p clinicore-worker` sob o `infisical run`
  liga o SMTP real, sem stub, e entrega tudo o que a API grava na fila. Teste à mão usa só endereço `@example.com`, que é
  reservado; `exemplo.com` não é.
- **O `openidconnect` 4.0.1 vem sobre o `oauth2` 5.0.0, que só aceita o `reqwest` 0.12, e é ele que a
  API usa**, pelo reexport `openidconnect::reqwest`. Um `reqwest` próprio no `Cargo.toml` põe duas
  versões no binário.
- **As chaves públicas do Google são baixadas a cada login**, e não ficam em env nem em cache: o Google
  as troca sozinho. O `CoreClient` nasce com um conjunto de chaves vazio, então o `id_token_verifier()`
  dele recusa todo token: o verificador é montado por login, com as chaves baixadas, e as chaves são
  baixadas antes de o `code` ser trocado, para uma falha nelas não gastar o `code`. Sem discovery, para o boot não depender do Google.
- **Os endpoints do Google são o `GoogleEndpoints`, e não variável de ambiente**: emissor, autorização,
  token e chaves são iguais em todo ambiente. O binário usa o `GoogleEndpoints::production()`, e o
  teste sobe um axum em `127.0.0.1:0`, que assina o `id_token` com a chave de teste de
  `tests/api/oauth/google_test_key.der`, e o passa ao `GoogleClient` pelo `app_with_google` do `support.rs`.
- **O bind em `::` entrega o IPv4 como `::ffff:a.b.c.d`.** O `ClientAddress` de `http/client.rs`
  passa o endereço por `to_canonical()`, e o limite agrupa o IPv6 em /64 antes de montar a chave.
- **O `MockConnectInfo` do axum não põe `ConnectInfo` nas extensions**, só o extractor o enxerga. O
  teste insere `ConnectInfo` direto na requisição, porque o `ClientAddress` lê as extensions.
- **Postgres e Redis conectam sob demanda.** O boot não espera nenhum dos dois, e com o Redis fora a
  rota limitada responde `503` enquanto o `/health` continua `200`.
- **O teste de boot roda o binário com `env_clear()`**, para não enxergar o ambiente de quem roda a
  suíte.
- **O `sqlx` liga a feature `log` do `tracing`, e com ela cada campo calculado dentro de um
  `tracing::info!` vira uma região que nunca roda**: a do fallback para o `log`, que só existe quando
  nenhum subscriber está ligado. O valor é calculado numa variável antes da macro, e a macro recebe só
  o nome (`tracing::info!(status, duration_ms, "request")`).
- **O `llvm-cov report` conta uma função genérica pela instanciação mais coberta**, e não pela união
  delas. Um `ValidJson<SignUpRequest>` que só vê o `415` noutro tipo de corpo deixa a região aberta, então
  cada caminho da função genérica precisa rodar na mesma instanciação.
- **Perfil velho do `llvm-cov` distorce a medida.** Antes de medir de novo, `cargo llvm-cov clean
  --workspace`.

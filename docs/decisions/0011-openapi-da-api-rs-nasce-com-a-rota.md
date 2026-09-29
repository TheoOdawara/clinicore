# 0011. OpenAPI da `api-rs` nasce com a rota

- Status: accepted
- Date: 2026-09-29
- Complementa: `0009-api-em-rust-com-axum-e-sqlx.md` e `0010-api-rs-com-crate-por-processo.md`, no
  documento servido em `/api-json`.

## Context

O documento OpenAPI é contrato: o `swagger_parser` do `apps/mobile` gera dele o cliente Retrofit, e
quem integra lê o `/api`. Na `api-rs` ele vinha de duas listas separadas: o `Router` de cada feature
registrava a rota, e o `http/openapi.rs` repetia à mão os handlers no `#[openapi(paths(...))]`. Uma
rota nova sem a segunda linha funcionava e sumia do documento, sem teste nem compilador que
acusassem.

A auditoria da sessão (#114) achou o documento mais pobre que a API:

- **Nenhum erro tinha schema.** O `Problem` e o `FieldError` do `http/error.rs` não eram `ToSchema`,
  então o cliente gerado não tinha modelo para ler o corpo de um `4xx`.
- **O `Clinicore-Client` era uma `String` livre.** A API aceita só `mobile` ou a ausência, e responde
  `400 invalid-client` a qualquer outro valor; o documento não dizia isso ao gerador.
- **O que o guard devolve não aparecia na rota.** 400 de validação ou de cliente, 403 de origem, 429 do
  limite e 503 do Redis fora são respostas de toda rota de negócio, e nenhuma as declarava.
- **O `Set-Cookie` e o `Location` não estavam documentados**, e o `201` do sign-in tinha dois corpos
  para um status só.

## Decision

- **A rota e a doc nascem na mesma linha, pelo `utoipa-axum`.** O `routes()` de cada feature devolve
  um `OpenApiRouter` montado com `routes!(handler)`, e o `lib.rs` junta os routers e separa o `Router`
  do documento com `split_for_parts`. O `http/openapi.rs` guarda só o `info` e o schema `Problem`, e
  perde a lista de `paths`. Rota registrada fora do `routes!` é proibida.
- **O limite por IP continua `route_layer` da rota**, aplicado no `MethodRouter` que o `routes!`
  devolve, com a mesma ordem da regra "cliente, origem, limite, validação" do `apps/api-rs/CLAUDE.md`.
- **O `Problem` e o `FieldError` são `ToSchema`, com o nome do tipo Rust** (ADR 0010). Toda resposta de
  erro documentada tem corpo `Problem` em `application/problem+json`.
- **O que os guards fazem é documentado por uma função só, `http::openapi::document_guards`**, aplicada
  ao documento das rotas de negócio antes de juntá-lo ao do `/health`. Ela põe em toda operação o
  header `Clinicore-Client`, um enum com o único valor `mobile`, e as respostas 400, 429 e 503, mais a
  403 em método que muda estado. A resposta que a rota já declara vence a da função, então a rota
  que devolve outro slug no mesmo status escreve a lista inteira. O `/health` fica sem nada disso: é
  lido por monitor, que não manda o header e não passa por limite.
- **O slug possível em cada status vai no `description` da resposta, como texto.** Nenhum tipo novo
  por slug, porque o gerador lê só o schema do corpo.
- **Cada status tem um corpo só.** O `201` do sign-in é o `SignInResponse` com `tokens` opcional, ausente
  no web. O `Set-Cookie` é header documentado nas respostas do web, e o `Location` no `201`.

## Consequences

Fica mais fácil:

- Rota nova aparece no documento sem passo a mais, e rota sem `#[utoipa::path]` não compila dentro do
  `routes!`.
- O cliente Dart ganha o modelo `Problem` para ler o `type` do erro e o enum do `Clinicore-Client`.
- As respostas dos guards mudam num lugar só quando um guard novo entrar, e rota nova as recebe sem
  declarar.

Fica mais difícil, e é aceito:

- **Uma dependência a mais**, o `utoipa-axum`, que segue a versão do `utoipa`. Ele sobe junto quando
  o `utoipa` subir.
- **Os routers das features passam a ser `OpenApiRouter`**, e método do axum que ele não repassa exige
  converter para `Router` antes.
- **O cliente Dart muda de forma**: nomes da ADR 0010, `Problem`, o enum e o `tokens` opcional. A #114
  o regenera junto com esta decisão.
- **Campo de request com `#[serde(default)]` sai opcional no schema**, e o cliente gerado o tipa como
  `String?`. Ele leva `#[schema(required = true)]`.

## Alternatives considered

- **Um `IntoResponses` e um `IntoParams` derivados, citados em cada rota** · rejeitada: o tipo existe só
  para o derive, nunca é construído, e o `rustc` acusa `dead_code`; além disso, cada rota nova teria de
  lembrar de citá-los.
- **Manter a lista manual e um teste que compara as rotas com os `paths`** · rejeitada: o teste precisa
  de uma terceira lista, a das rotas esperadas, e o erro continua sendo possível até ele rodar.
- **Um tipo de resposta por slug de erro** · rejeitada: o `swagger_parser` não tipa corpo de erro por
  rota, todo erro vira `DioException`, e o app decide pelo `type` do `Problem`. O tipo não teria leitor.
- **`oneOf` ou dois content-types no `201` do sign-in** · rejeitada: o gerador faz um tipo pior de usar
  que um campo opcional, e o web ignora o corpo além do `user`.
- **Escrever o OpenAPI à mão num YAML** · rejeitada: volta a ter duas fontes, a rota e o documento.

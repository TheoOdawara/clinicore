# 0010. API em Rust com um crate por processo e camada só quando tem conteúdo

- Status: accepted
- Date: 2026-09-29
- Substitui: `0009-api-em-rust-com-axum-e-sqlx.md`, só na organização do workspace. A troca de stack da
  0009 (axum, sqlx, `utoipa`, `tracing`, `lettre` e o resto da tabela "Sai / Entra") continua valendo.

## Context

A 0009 organizou o workspace com um crate por camada, `http → app → infra`, para o compilador impedir
um handler de chamar um repository. Com a primeira feature portada, a #117, a organização se mostrou
cara de navegar: uma operação atravessa três crates, cada feature se repete com o mesmo nome em três
lugares, e uma consulta simples exige handler, service e repository mesmo quando o service só repassa
a chamada.

O produto ainda vai ter mais de um processo em Rust: o worker da fila (a #71 é o primeiro job) e a
importação e exportação de arquivos da portabilidade, que o `docs/requirements.md` deixa fora do
go-live. É isso que merece fronteira de crate. Camada não merece.

## Decision

**Um crate por processo ou capacidade, cada um numa pasta de `apps/api-rs/crates/`:**

| Pasta | Pacote | Tipo | Contém |
| --- | --- | --- | --- |
| `crates/core/` | `clinicore-core` | biblioteca | configuração, pool do Postgres, Redis, e-mail e o layout dele |
| `crates/app/` | `clinicore-app` | binário `api` e biblioteca | a API inteira: rotas, handlers, services, consultas, erro, telemetria, OpenAPI |
| `crates/worker/` | — | binário | nasce com o primeiro job da fila |
| `crates/transfer/` | — | biblioteca | nasce com a feature de portabilidade |

- **O pacote tem o prefixo `clinicore-`.** `core` é o nome da biblioteca base do Rust, e `app` é
  genérico demais para `cargo -p`.
- **Crate só nasce com código.** `worker/` e `transfer/` não existem até ter o primeiro arquivo que
  justifique cada um.
- **Dentro do `app`, a feature é um módulo, e a camada nasce quando tem conteúdo.** Uma leitura
  simples vai do handler direto à consulta. O service existe quando há decisão: regra de negócio,
  orquestração de mais de uma escrita ou efeito colateral. Um arquivo novo nasce de responsabilidade
  misturada, nunca de um padrão a cumprir.
- **SQL só em `queries.rs` da feature**, por `query!` ou `query_as!`, e transação só lá. SQL montado
  por `format!` ou concatenação continua proibido.
- **O `AppError` do `app` é um `enum` do `thiserror` e implementa o `IntoResponse` direto.** O
  newtype `ApiError(AppError)` da 0009 existia só por causa da orphan rule entre dois crates, e deixa
  de ser necessário. Cada variante sabe o próprio status, o `sqlx::Error` entra pelo `#[from]`, e a
  rejeição do `ValidJson` é o mesmo `AppError`.
- **O código de campo é o do `validator`** (`email`, `length`, `weak_password`), e campo desconhecido
  ou de tipo errado sai como `invalid`. A 0009 mandava repetir os códigos da implementação anterior;
  nenhum cliente os lê, e a forma do Rust vence.
- **O schema do OpenAPI tem o nome do tipo Rust** (`SignUpRequest`, `EmailRequest`), sem
  `#[schema(as = …)]`. O cliente Dart muda de nome quando for regenerado, na #121.

## Consequences

Fica mais fácil:

- Tudo de uma feature fica numa pasta só, `crates/app/src/<feature>/`.
- Uma leitura simples tem um arquivo de SQL e um handler, sem service de repasse.
- O worker e o importador entram como crates novos que dependem do `core`, sem mexer na API.

Fica mais difícil, e é aceito:

- **O compilador deixa de impor as camadas.** O `PgPool` é público no `core`, e um handler consegue
  rodar SQL. A regra "SQL só em `queries.rs`" passa a ser convenção, conferida no review.
- **Uma consulta que o worker também precise** fica repetida nele ou desce para o `core`. A escolha é
  feita quando o worker nascer.
- **O `app` inteiro recompila a cada mudança**, o que a 0009 rejeitou. Hoje compila em segundos; a
  divisão volta a ser discutida quando o tempo medido pedir.

## Alternatives considered

- **Manter a 0009** · rejeitada: a trava do compilador não compensa três crates por feature, nem o
  service de repasse numa leitura simples.
- **Crates por processo com `http`, `app` e `infra` dentro de cada um** · rejeitada: repete o custo da
  0009 em cada processo.
- **Consultas no `core`, compartilhadas desde já** · rejeitada: o único consumidor hoje é a API, e o
  worker ainda não existe para dizer do que precisa.

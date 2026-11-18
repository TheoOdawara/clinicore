---
name: gates
description: Os gates de cada app do Clinicore, com o comando exato e a ordem. Use antes de dar qualquer mudança por pronta, e para saber o que o CI vai rodar.
---

# gates

Cada comando roda de dentro do diretório do app, do mais barato ao mais caro. Um gate que falha para a
entrega: o resultado é relatado com a saída.

## `apps/api`

O Postgres e o Redis de dev sobem com `infisical run --path=/api -- docker compose up -d --wait`. As
migrations rodam antes do clippy: as macros do sqlx conferem cada consulta contra o banco.

| Gate | Comando |
| --- | --- |
| Formato | `cargo fmt --check` |
| Dependências e CVE | `cargo deny check` |
| Schema | `infisical run --path=/api -- sqlx migrate run` · `infisical run --path=/api -- cargo sqlx prepare --workspace --check` |
| Análise estática e tipos | `infisical run --path=/api -- cargo clippy --all-targets -- -D warnings` |
| Build | `infisical run --path=/api -- cargo build` |
| Testes | `infisical run --path=/api -- cargo llvm-cov nextest --lcov --output-path lcov.info` |
| Cobertura das linhas novas | da raiz do repo: `uvx diff-cover==11.0.3 apps/api/lcov.info --compare-branch=origin/staging --fail-under=90` |
| Cobertura dos caminhos críticos | o mesmo `diff-cover` com `--fail-under=100 --include` e os caminhos críticos do `apps/api/AGENTS.md`, cada um prefixado com `apps/api/` |

O `sqlx-cli` 0.9.0 é instalado com
`cargo install sqlx-cli --version 0.9.0 --locked --no-default-features --features postgres,rustls`, e o
`cargo-nextest`, o `cargo-llvm-cov` e o `cargo-deny` com `cargo install --locked`. O `diff-cover`
compara com a `origin/staging`, então um `git fetch origin staging` vem antes. Ele roda da raiz do repo:
o `--include` casa com o caminho a partir da raiz, e rodado de dentro do app não casa nada.

## `apps/web`

Hoje é o scaffold em React; os gates abaixo são os dele, e mudam na migração para o Angular.

| Gate | Comando |
| --- | --- |
| Instalação | `bun install --frozen-lockfile` |
| CVE | `bun audit` |
| Lint | `bun run lint` |
| Formato | `bun run format:check` |
| Build | `bun run build` |
| Tipos | `bun check` |
| Testes | `bun run test` |

O scaffold ainda tem o limite de 100% no `vite.config.ts`; ele sai na migração, pela ADR 0023.

## `apps/site`

Hoje é o scaffold em Next; os gates mudam na migração para o SvelteKit.

| Gate | Comando |
| --- | --- |
| Instalação | `bun install --frozen-lockfile` |
| CVE | `bun audit` |
| Lint | `bun run lint` |
| Formato | `bun run format:check` |
| Build | `bun run build` |
| Tipos | `bun check`, depois do build |
| Conteúdo do prerender | `grep -Eq '<h1[^>]*>Clinicore</h1>' .next/server/app/index.html` |
| Testes | `bun run test` |

## `apps/android` e `apps/ios`

n/a: os apps ainda não existem. Os gates entram nesta skill com o scaffold de cada um, na forma da ADR
0022 (Spotless, detekt, Android Lint, Konsist, Kover e osv-scanner no Android; swift-format, SwiftLint
`--strict`, Swift Testing e XCUITest no iOS).

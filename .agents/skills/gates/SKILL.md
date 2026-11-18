---
name: gates
description: Os gates de cada app do Clinicore, com o comando exato e a ordem. Use antes de dar qualquer mudança por pronta, e para saber o que o CI vai rodar.
---

# gates

Cada comando roda do diretório indicado, do mais barato ao mais caro. Um gate que falha para a entrega:
o resultado é relatado com a saída.

## Documentação

Da raiz do repo:

| Gate | Comando |
| --- | --- |
| Build do site | `uvx zensical==0.1.2 build --clean` |

## `apps/api`

De `apps/api/`, com o compose de pé (`infisical run --path=/api -- docker compose up -d --wait`). O CI
roda os mesmos, no `ci-api.yml`.

| Gate | Comando |
| --- | --- |
| Formato | `cargo fmt --all --check` |
| Dependências | `cargo deny check` |
| Lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Consultas offline | `infisical run --path=/api -- cargo sqlx prepare --workspace --check` |
| Testes e cobertura | `infisical run --path=/api -- cargo llvm-cov nextest --workspace --locked --lcov --output-path target/lcov.info --ignore-filename-regex 'main\.rs'` |
| Linhas novas | da raiz: `uvx diff-cover==11.0.3 apps/api/target/lcov.info --compare-branch origin/staging --include-untracked --fail-under 90`, e a mesma chamada com `--fail-under 100` e o `--include` dos caminhos críticos do `ci-api.yml` |

## `apps/web`

De `apps/web/`, depois do `bun install`. O CI roda os mesmos, no `ci-web.yml`.

| Gate | Comando |
| --- | --- |
| Vulnerabilidades conhecidas | `bun run audit` |
| Formato | `bunx prettier --check .` |
| Lint e fronteiras | `bun run lint` |
| Tipos dos arquivos de configuração e do e2e | `bun run typecheck` |
| Teste das fronteiras | `bun run test:boundaries` |
| Build | `bun run build` |
| Testes e cobertura | `bun run test -- --coverage --watch=false` |
| E2E, a 390 px e com o axe | `bun run e2e` |
| Linhas novas | da raiz: `uvx diff-cover==11.0.3 apps/web/coverage/web/lcov.info --compare-branch origin/staging --include-untracked --fail-under 90`, e a mesma chamada com `--fail-under 100` e o `--include` dos caminhos críticos do `ci-web.yml` |

## Os outros apps

Ainda sem código. Os gates de cada um entram nesta skill com o scaffold dele, como comandos que o
scaffold faz passar, e cada app ganha o seu `ci-<app>.yml`.

| App | Gates | Ferramentas decididas |
| --- | --- | --- |
| `apps/site` | n/a: sem código | ESLint com `eslint-plugin-svelte` e `eslint-plugin-boundaries`, Prettier, `svelte-check`, `vite build`, Vitest (ADRs 0021, 0026) |
| `apps/android` | n/a: sem código | Spotless, detekt, Android Lint, Konsist, Kover, osv-scanner (ADRs 0022, 0026) |
| `apps/ios` | n/a: sem código | swift-format, SwiftLint `--strict`, Swift Testing, XCUITest (ADRs 0022, 0026) |

Em todo app, a cobertura das linhas novas é conferida pelo `diff-cover` contra a `origin/staging`, como a
skill `testing` descreve (ADR 0023).

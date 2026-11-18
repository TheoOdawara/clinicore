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

## Apps

Nenhum app tem código ainda, então nenhum gate de app roda. Os gates de cada um entram nesta skill com o
scaffold dele, como comandos que o scaffold faz passar, e cada app ganha o seu `ci-<app>.yml`.

| App | Gates | Ferramentas decididas |
| --- | --- | --- |
| `apps/api` | n/a: sem código | `cargo fmt`, `cargo deny`, `sqlx prepare --check`, `cargo clippy -D warnings`, `cargo llvm-cov nextest` (ADRs 0009, 0019) |
| `apps/web` | n/a: sem código | angular-eslint com `eslint-plugin-boundaries`, Prettier, `ng build`, Vitest pelo `ng test`, Playwright (ADRs 0020, 0026) |
| `apps/site` | n/a: sem código | ESLint com `eslint-plugin-svelte` e `eslint-plugin-boundaries`, Prettier, `svelte-check`, `vite build`, Vitest (ADRs 0021, 0026) |
| `apps/android` | n/a: sem código | Spotless, detekt, Android Lint, Konsist, Kover, osv-scanner (ADRs 0022, 0026) |
| `apps/ios` | n/a: sem código | swift-format, SwiftLint `--strict`, Swift Testing, XCUITest (ADRs 0022, 0026) |

Em todo app, a cobertura das linhas novas é conferida pelo `diff-cover` contra a `origin/staging`, como a
skill `testing` descreve (ADR 0023).

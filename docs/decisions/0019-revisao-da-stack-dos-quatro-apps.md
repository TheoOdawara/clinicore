# 0019. Revisão da stack dos quatro apps

- Status: accepted
- Date: 2026-10-25
- Substitui: 0004, 0005 e 0015 inteiras
- Emenda a: 0003 (a biblioteca de animação), 0007 (o cliente HTTP gerado, o estado, os gates e os testes
  do mobile), 0014 (as imagens base de cada Dockerfile) e 0011 (quem consome o OpenAPI)

## Context

A #154 levaria o `apps/web` e o `apps/site` para o Bun, como a 0015 decidiu. A 0015 nunca foi
implementada, e a issue virou a revisão da stack dos quatro apps, um app por vez.

Em 2026-10-25, as dependências travadas no repo tinham CVE aberta:

- **`apps/site`:** `next` 16.3.6, com 10 advisories, entre elas duas execuções remotas de código, uma delas
  na otimização de imagem que o `next/image` usa. Também `sharp`, abaixo de 0.35.5, e `source-map-js`.
- **`apps/web`:** `vite` 8.3.0, `zod` abaixo de 4.6.6 e `source-map-js`.
- **`apps/api`:** `tokio` 1.53.1 (RUSTSEC-2026-0386), `ctutils` 0.4.2 (RUSTSEC-2026-0385) e `crypto-bigint`
  0.5.5 (RUSTSEC-2026-0387).

Além disso, nenhum app tinha exigência de cobertura, e o mobile gerava o cliente HTTP a partir do
OpenAPI da API.

## Decision

**Geral**

- Toda dependência fica na última versão estável, fixada sem `^` e sem `~`. Cada app tem um gate de CVE no
  CI.
- **Cada app exige 100% de cobertura**, somando teste unitário e de integração. Só ficam de fora o código
  gerado e os arquivos que só declaram tipo. Componente copiado, do shadcn ou do React Bits, conta.
- O e2e roda contra a API real: Playwright no web e Patrol no mobile. Ele entra com a primeira tela, na
  #78 e na gêmea dela no mobile.

**`apps/site`**

| Papel | Escolha |
| --- | --- |
| Runtime | Node 26.12.1, para `next dev`, `next build` e o `server.js` do standalone |
| Pacotes | Bun 1.4.4: `bun.lock`, `[install] exact = true` e `packageManager` |
| Tipos | TypeScript 7.0.2 · `bun check` |
| Framework | Next 16.4.3 com App Router e `output: "standalone"` · React 19.3.1 |
| Lint | oxlint type-aware, com os plugins `typescript`, `react`, `nextjs` e `jsx-a11y` |
| Formato | `bun fmt` |
| Testes | Vitest com 100% sobre componente cliente e lógica. As páginas e os layouts de `src/app/` ficam com o gate de prerender |
| Estilo | Tailwind 4.3 com `@tailwindcss/postcss`, que usa o Lightning CSS internamente, assim como o Turbopack |
| Componentes | shadcn sobre Radix, com `lucide-react`, a partir do primeiro componente |
| Animação | CSS nativo e Motion. GSAP e three.js só entram se uma seção desenhada pedir |
| Analytics | Cloudflare Web Analytics, que não usa cookie e dispensa banner, a partir do DNS |
| Imagem | instala com `oven/bun:1.4.4` e roda em `node:26.12.1-slim` |

**`apps/web`**

| Papel | Escolha |
| --- | --- |
| Runtime e pacotes | Bun 1.4.4, com `bun --bun` em dev, build e teste |
| Tipos | TypeScript 7.0.2 · `bun check` |
| Framework | React 19.3.1 · Vite 8.3 · TanStack Router por arquivo |
| Lint e formato | oxlint type-aware (`typescript`, `react`, `jsx-a11y`) · `bun fmt` |
| Testes | Vitest com happy-dom, em ordem aleatória, cobertura v8 com threshold de 100. Na #78 entram Testing Library, MSW renderizando a rota de verdade e Playwright |
| Animação | Motion, instalado já. React Bits segue a tabela da 0003, com preferência pela variante em Motion |
| Dados, na #78 | TanStack Query · axios · TanStack Form · Zod |
| Imagem | build em `oven/bun:1.4.4` e o `dist/` servido por nginx |

**`apps/api`**

| Papel | Escolha |
| --- | --- |
| Toolchain | Rust 1.99.1 |
| Testes | `cargo nextest run` · `cargo llvm-cov nextest` com 100 em linhas, funções e regiões |
| Auditoria | `cargo-deny`, com advisories, licenças, crates duplicados e origens |
| Banco e cache | `postgres:18.6-alpine` · `redis:8.10.2-alpine`, com a tag exata |
| Build sem banco | `SQLX_OFFLINE=true`, com o `.sqlx/` commitado |
| Imagem | build em `rust:1.99.1-slim` e runtime em `debian:trixie-slim` com tag datada e `ca-certificates`, com os binários `api` e `worker` |

**`apps/mobile`**

| Papel | Escolha |
| --- | --- |
| SDK | Flutter 3.47.8 · Dart 3.13.7 |
| HTTP | `dio` direto, sem cliente gerado. Os modelos são escritos à mão, com `fromJson` por pattern matching |
| Estado | Riverpod com `riverpod_generator`, a partir da primeira tela |
| Lint | `very_good_analysis`, com `strict-casts`, `strict-inference` e `strict-raw-types` |
| Auditoria | `osv-scanner` sobre o `pubspec.lock` |
| Testes | `flutter_test` com `mocktail` e `http_mock_adapter` · `very_good test --min-coverage 100` · Patrol em Android e iOS no CI |

## Consequences

- **O site tem dois runtimes.** O Bun instala e roda as ferramentas, e o Node roda o Next. No CI, o job
  `site` usa o `setup-bun` e o `setup-node`. O web não tem Node.
- **O oxlint cobre o que a 0004 exigia.** Conferido com o oxlint 1.89.0: o `typescript/no-deprecated`
  reprova `"abc".substr(1)`, e o `nextjs/no-img-element` reprova `<img>`. As pegadinhas de ordem de config
  do `eslint-config-next` deixam de existir.
- **O `bun check` também é um script do `package.json`.** Por isso nenhum app pode ter um script chamado
  `check`, que passaria na frente do type checker.
- **No web, o teste lê o `import.meta.env` como o build lê.** O Vitest usa o `vite.config.ts`, e a
  pegadinha do Jest com `import.meta.env` deixa de existir. Conferido com o Vitest 5.0.5 sob
  `bun --bun`: o teste preenche um formulário pelo rótulo com `user-event` e confere a chamada com
  `vi.fn`. A cobertura v8 reprova código não executado.
- **O happy-dom não é o jsdom.** Uma API do DOM que ele não implemente quebra o teste. Isso não acontece
  no navegador. O jsdom fica de fora porque, em 2026-10-13, não iniciava o worker do Vitest sob
  `bun --bun`.
- **O Next 16.4.3 builda com o TypeScript 7.0.2.** Conferido no Node: a checagem de tipos do próprio
  `next build` passa.
- **O RUSTSEC-2026-0387 fica ignorado no `deny.toml`, com o motivo.** O `crypto-bigint` 0.5.5 vem pelo
  `p256` 0.13, que o `jsonwebtoken` 11.1.0 e o `openidconnect` 4.0.1 trazem. Os dois já estão na última
  versão. O código não passa pelo `p256`: o access token é HS256, e o `id_token` do Google é RS256. A
  exceção sai quando os dois atualizarem o `p256`.
- **O Rust stable não mede branch.** O `--branch` do `llvm-cov` exige nightly, então a cobertura por
  região é a medida mais próxima disponível.
- **O nextest não roda doctest.** O repo não tem nenhum, porque o contrato só aceita comentário
  `shortcut:`. Por isso o `cargo test` sai dos gates.
- **O mobile replica o DTO à mão, como o web já fazia com o Zod.** O OpenAPI da 0011 serve só para gerar
  o Swagger UI, e nenhum app gera código a partir dele. O `build_runner` fica no mobile por causa do drift
  e do Riverpod.
- **O CI ganha um job macOS** para o Patrol no simulador iOS, quando ele entrar. O runner macOS do GitHub
  não tem Docker, então o Postgres e o Redis desse job sobem pelo Homebrew, e a API compila no próprio
  macOS.

## Alternatives considered

- **`bun test` em vez do Vitest, como a 0015 decidiu** · rejeitada pelo dono do produto.
- **Bun como runtime do site** · rejeitada pelo dono do produto. O Next roda no Node, que é o runtime
  que ele suporta.
- **ESLint e Prettier** · rejeitados pelo dono do produto, em favor do oxlint e do `bun fmt`.
- **GSAP como biblioteca de animação do site** · rejeitada. O Motion é o padrão no React, e o GSAP só se
  justifica numa seção fixada com timeline longa.
- **Google Analytics** · rejeitado. Ele grava o cookie `_ga`, e a LGPD exige banner de consentimento.
- **`vi.mock` nas chamadas da feature em vez do MSW** · rejeitado. Com ele, o caminho do axios até a tela
  nunca roda junto.
- **`cargo audit`** · rejeitado. O `cargo-deny` faz a mesma auditoria e ainda confere licença e origem.
- **Runtime distroless para a API** · rejeitado pelo dono do produto, em favor do `debian:trixie-slim`.
- **Bloc no mobile** · rejeitado. O Riverpod é o mais próximo do TanStack Query do web.
- **Cliente gerado do OpenAPI no mobile (0007)** · rejeitado pelo dono do produto.
- **iOS só testado à mão** · rejeitado pelo dono do produto.

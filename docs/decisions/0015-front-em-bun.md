# 0015. Front em Bun

- Status: superseded by 0019
- Date: 2026-10-19
- Substitui: 0005

## Context

A ADR 0005 tirou o Bun do `apps/web` porque ele nunca ficava sozinho: o binário do Jest é
`#!/usr/bin/env node`, e o front precisava de dois runtimes, em cada máquina e no job `web` do CI.

Desde então, duas coisas mudaram:

- **A API deixou de ser Node.** A ADR 0009 levou o `apps/api` para Rust. O Node ficou no repo só por
  causa do web e do site.
- **O runner de teste pode ser o do próprio Bun.** Conferido em 2026-10-19 com o Bun 1.4.3: o `bun test`,
  com o `@happy-dom/global-registrator` 21.0.3 pré-carregado pelo `bunfig.toml`, renderiza um componente
  React 19.3 com o `@testing-library/react` 16.3.3, acha o campo pelo rótulo, dispara o submit e confere
  a chamada com o `mock` do `bun:test`.

O site também roda sem Node. Conferido em 2026-10-15 com o Bun 1.4.3 e o `next@16.3.6`: o standalone
gerado por `bun --bun next build` e servido por `bun server.js` devolve `image/webp` pelo `next/image`
e pelo `sharp`, localmente e nas imagens `oven/bun:1.4.3-slim` e `oven/bun:1.4.3-alpine`.

O `apps/web` ainda tem uma suíte só, sem nenhum componente testado. O `apps/site` não tem teste.

## Decision

**O `apps/web` e o `apps/site` passam para o Bun, como gerenciador de pacotes e como runtime.** O Node
sai do repo.

- O `bun.lock` entra no lugar do `package-lock.json`, nos dois apps.
- Os scripts de dev, build e lint rodam sob `bun --bun`, e o `engines` dos dois `package.json` passa a
  declarar o Bun.
- **O gate de tipos é o `bun check`**, o type checker nativo do Bun, nos dois apps. O script
  `typecheck` sai, e o script `check` também, porque um script com esse nome tem precedência sobre o
  type checker. O pacote `typescript` fica, porque o `typescript-eslint` e o editor dependem dele.
- O `bunfig.toml` de cada app declara `[install] exact = true`, para que o `bun add` grave a versão
  exata, sem `^`.
- O CI instala com `bun install --frozen-lockfile` e audita com `bun audit`.
- O `apps/web` testa com `bun test`, com o `@happy-dom/global-registrator` pré-carregado pelo
  `bunfig.toml` e `[test] randomize = true`. O Jest, o `@swc/jest`, o `jest-environment-jsdom` e o
  `jest.config.json` saem.
- Os jobs `web` e `site` do `ci.yml` usam o `oven-sh/setup-bun`, sem o `actions/setup-node`.
- O Dockerfile do `apps/site`, previsto na ADR 0014, parte da imagem `oven/bun`.

## Consequences

- **Um runtime só no front, e nenhum Node no repo.** O que a ADR 0005 queria com o npm, um runtime por
  máquina, fica valendo do outro lado.
- **O `jest.fn` vira o `mock` do `bun:test`, e o mock de módulo vira o `mock.module`.** Quem escreve o
  primeiro teste de componente (a #78) já escreve assim.
- **O import de um `.module.css` chega como objeto vazio no `bun test`.** A classe de um componente é
  `undefined` na suíte, então nenhum teste afirma sobre classe CSS.
- **O happy-dom não é o jsdom.** Uma API do DOM que o happy-dom não implementa quebra o teste, e não o
  navegador. É o preço de um runtime só.
- **Um script `check` no `package.json` esconde o type checker:** o `bun check` roda o script quando ele
  existe. Nenhum dos dois apps pode ter um script com esse nome.
- **O type checker do Bun respeita o `tsconfig.json`.** Conferido em 2026-10-19 com o Bun 1.4.3 numa cópia
  do `apps/web`: ele reprovou TS2322, TS7006 e TS6133, então `strict` e `noUnusedLocals` valem, em
  cerca de 50 ms contra 0,9 s do `tsc`.
- **Os testes rodam em ordem aleatória**, então um teste que depende de outro falha logo.
- **O `minimumReleaseAge` fica desligado**, porque atrasaria a última versão estável que o repo usa.
- **A licença e o diff de uma dependência nova ou atualizada saem do próprio Bun**, pelo
  `bun pm licenses` e pelo `bun pm diff`.
- **Os comandos mudam em todo lugar que os cita:** o `AGENTS.md` da raiz e o de cada app, o README, o
  `.vscode/` e o CI, todos na #154.

## Alternatives considered

- **Manter o npm sobre o Node (ADR 0005)** · rejeitada pelo dono do produto.
- **O Bun só como gerenciador de pacotes, com o Node executando** · rejeitada: volta a ter dois runtimes,
  que é o problema que a ADR 0005 resolveu.
- **Vitest com jsdom no lugar do `bun test`** · rejeitada: é mais um runner para manter, e em 2026-10-13,
  com o Bun 1.4.3, o jsdom já falhou ao iniciar o worker do Vitest sob `bun --bun`, num projeto onde o
  happy-dom passava os mesmos testes.

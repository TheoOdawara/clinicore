# 0015. Front em Bun

- Status: accepted
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
- O `apps/web` testa com `bun test`, com o `@happy-dom/global-registrator` pré-carregado pelo
  `bunfig.toml`. O Jest, o `@swc/jest`, o `jest-environment-jsdom` e o `jest.config.json` saem.
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
- **O `bun check` roda o script `check` do `package.json`**, que existe nos dois apps, e não o type
  checker do Bun. O gate de tipos continua sendo o `tsc` do script `typecheck`.
- **Os comandos mudam em todo lugar que os cita:** o `AGENTS.md` da raiz e o de cada app, o README, o
  `.vscode/` e o CI, todos na #154.

## Alternatives considered

- **Manter o npm sobre o Node (ADR 0005)** · rejeitada pelo dono do produto.
- **O Bun só como gerenciador de pacotes, com o Node executando** · rejeitada: volta a ter dois runtimes,
  que é o problema que a ADR 0005 resolveu.
- **Vitest com jsdom no lugar do `bun test`** · rejeitada: é mais um runner para manter, e em 2026-10-13,
  com o Bun 1.4.3, o jsdom já falhou ao iniciar o worker do Vitest sob `bun --bun`, num projeto onde o
  happy-dom passava os mesmos testes.

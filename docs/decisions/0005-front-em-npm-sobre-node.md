# 0005. Front em npm sobre Node 26

- Status: accepted
- Date: 2026-09-20
- Emenda a: 0002 (gerenciador de pacotes do front) e 0001 (a nota dos dois gerenciadores)

## Context

A ADR 0001 reconstruiu o `apps/api` em Node 26 e npm, e registrou que o repo passaria a ter dois
gerenciadores de pacote — npm na API, Bun no front — "até o web migrar para Next". A ADR 0002 decidiu
que o web não migra para Next e promoveu os dois gerenciadores de estado transitório a estado decidido.

O que essa promoção não pesou é que **o front já depende de Node de qualquer jeito**. O binário do Jest
é `#!/usr/bin/env node`: o `apps/web/CLAUDE.md` registra a pegadinha, e o job `web` do CI instala
`oven-sh/setup-bun` **e** `actions/setup-node` por causa dela. O Bun nunca ficou sozinho no front; ele
ficou por cima de um Node que tinha de estar lá.

O preço aparece em três lugares: dois runtimes para instalar e manter em cada máquina de
desenvolvimento, dois passos de setup no job `web` do CI, e duas formas de instalar — `npm ci` e
`bun install --frozen-lockfile` — para descrever em toda documentação que fala de comando.

O ganho que sobra é a velocidade de instalação. O `apps/web` tem hoje sete arquivos de código e 582
pacotes; o `npm install` dele leva trinta segundos numa árvore fria, e o CI ainda guarda cache por
`package-lock.json`. Nenhuma dependência do web ou do site exige Bun: o Vite, o Jest com `@swc/jest`,
o Tailwind e o TanStack Router rodam em Node.

O momento é o mesmo que a 0004 usou: o `apps/web` é pequeno e o `apps/site` ainda não existe.

## Decision

**O `apps/web` troca o Bun por npm sobre Node 26, e o `apps/site` nasce assim na #76.** O repo passa a
ter um gerenciador de pacotes só, nos três apps.

- `apps/web/bun.lock` sai do versionamento e no lugar entra `package-lock.json`.
- `apps/web/package.json` ganha `engines.node: ">=26"`, igual ao da API.
- O job `web` do `ci.yml` perde o `oven-sh/setup-bun` e instala com `npm ci`, sob
  `actions/setup-node@v6` com `cache: npm`.

**Os gates de lint e de formato passam a ser dois passos, nos dois jobs.** O `apps/web` ganha `lint` e
`format:check` com os mesmos nomes da API, e `check` vira o atalho local que encadeia os dois. Um gate
vermelho passa a dizer no nome do passo se caiu o ESLint ou o Prettier, sem abrir o log.

Esta decisão **não toca no ferramental de lint, na versão do TypeScript nem na escolha do Vite**: o que
muda é quem instala os pacotes e quem executa os scripts.

## Consequences

- **Um gerenciador de pacotes no repo inteiro.** A frase "a API usa npm e os dois apps de front usam
  Bun" sai do `CLAUDE.md` da raiz, junto da coluna de comandos do web e da linha de instalação.
- **O job `web` do CI perde um passo de setup** e passa a ter a mesma forma do job `api`.
- **Some a velocidade de instalação do Bun.** É o único ganho abandonado, e ele já era pago com um
  segundo runtime que o Jest obrigava a instalar do lado.
- **Entra um `package-lock.json` novo no versionamento**, com 582 pacotes. Ruído de diff pago uma vez.
- **A pegadinha "mesmo com Bun, o web precisa de Node instalado" deixa de existir**, e sai do
  `apps/web/CLAUDE.md`.
- **A #76 nasce com npm**, não com Bun, e o job `site` precisa só do `setup-node`.

## Alternatives considered

- **Manter o Bun no front** · rejeitada pelo dono do produto: manter dois runtimes por causa dos testes
  é custo recorrente de máquina e de documentação, contra um ganho de segundos numa instalação fria.
- **Levar a API para Bun e unificar do outro lado** · rejeitada: a ADR 0001 já decidiu Node e npm na
  API, com o NestJS, o TypeORM e a CLI de migration como razão. Reabrir isso trocaria um gerenciador
  por uma reescrita.
- **Deixar o site em Bun e migrar só o web** · rejeitada: deixaria no repo a mesma pendência que esta
  ADR fecha, e o site ainda não tem uma linha escrita para justificar a exceção.

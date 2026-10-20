# 0016. Front em CSS puro, com CSS Modules

- Status: accepted
- Date: 2026-10-19
- Substitui: 0003 na parte de estilo e de componentes base; a parte do React Bits continua valendo

## Context

A ADR 0003 escolheu o shadcn/ui sobre Radix, Tailwind 4 e CVA como design system do `apps/web` e do
`apps/site`. Nenhum componente dele chegou a ser copiado: hoje os dois apps têm o Tailwind 4.3 instalado,
uma rota cada um e nenhuma tela.

O que a plataforma já entrega sem o Tailwind:

- **CSS Modules** no Vite 8 e no Next 16, sem configuração.
- **Lightning CSS** no build: o Vite 8 minifica e rebaixa o CSS com ele, e o Next 16 com Turbopack sempre
  o usa.
- **Nesting nativo e `@layer`**, que o Lightning CSS rebaixa para os navegadores que ainda não os têm.

O `@eslint/css` 2.0.1 roda no ESLint 9.39.5 dos dois apps, conferido em 2026-10-19: ele reprova uma
propriedade desconhecida e aceita o nesting nativo pela regra de baseline.

## Decision

**O `apps/web` e o `apps/site` escrevem CSS puro, sem Tailwind e sem shadcn/ui.**

- **O `src/styles/globals.css` é o manifesto das camadas**, na ordem `@layer reset, tokens, base,
  components`, e cada camada mora no próprio arquivo, importado por ele.
- **Cada componente tem o seu `.module.css`**, ao lado do `.tsx`, com as classes em camelCase e sem BEM.
- **O HTML nativo vem primeiro.** Quando ele não basta, a primitiva é o Base UI (`@base-ui/react`), que
  entra com o primeiro componente que precisar dele. Data e hora, na agenda, usam o React Aria
  Components.
- **O lint de CSS é o `@eslint/css`**, com a regra `use-baseline`, no script `lint` de cada app.
- **O Tailwind sai dos dois apps:** as dependências, o `@tailwindcss/vite`, o `postcss.config.mjs` do
  site e a extensão recomendada no `.vscode/`.

**O React Bits da ADR 0003 continua**, com a tabela de categorias e as regras 1, 2 e 4 dela. Ele é
copiado na variante em CSS, nunca na de Tailwind. A regra 3 passa a ser "antes de copiar, checa-se se
uma regra CSS resolve".

## Consequences

- **A acessibilidade da regra 8 da spec `003-autenticacao-web.md` é escrita no projeto**: `label`
  associado, `aria-describedby`, `role="alert"` e `:focus-visible`. O Radix entregava isso pronto. Um
  componente de campo em `apps/web/src/shared/` resolve uma vez, e toda tela usa ele.
- **`&__campo` no nesting nativo compila sem erro e gera um seletor que nunca casa.** É o motivo de não
  haver BEM, e entra como pegadinha no `AGENTS.md` de cada app na #77.
- **Nenhuma dependência de estilo em runtime**, e nenhuma classe utilitária no JSX.
- **O tema, as cores e a tipografia ficam nos tokens da camada `tokens`.** Eles nascem na fase de design
  system, e o `docs/design/tokens.md` documenta os mesmos valores.
- **O teste não vê classe CSS:** no `bun test` da ADR 0015, o import de um `.module.css` chega vazio.

## Alternatives considered

- **Manter o shadcn/ui sobre Tailwind (ADR 0003)** · rejeitada pelo dono do produto.
- **Tailwind sem o shadcn** · rejeitada pelo dono do produto, junto com o shadcn.

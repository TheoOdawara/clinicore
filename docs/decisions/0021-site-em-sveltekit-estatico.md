# 0021. Site em SvelteKit estático

- Status: accepted, emendada pela 0026 na estrutura e nas convenções
- Date: 2026-11-17
- Substitui: 0008 inteira, 0019 na tabela do `apps/site` e 0018 no `apps/site`
- Emenda a: 0002 (o site deixa de ser Vite com prerender), 0003 (o site não usa shadcn/ui nem React
  Bits) e 0014 (a imagem do site passa a ser só nginx, sem Node em produção)

## Context

O `apps/site` é a landing pública em `clinicore.com.br`. Ele tem uma página e nenhum conteúdo dinâmico,
mas rodava um servidor Node em produção por causa do `output: "standalone"` do Next (0008), e a 0019
registrou dez advisories abertas no Next que ele usava, duas delas de execução remota de código.

O dono do produto decidiu levar o site para o SvelteKit, com as versões conferidas no registro do npm em
2026-11-17.

## Decision

**O `apps/site` é SvelteKit com `adapter-static`: tudo é prerenderizado no build, e o nginx serve os
arquivos.** Se um dia o site tiver conteúdo dinâmico, ele troca para o `adapter-node`.

| Papel | Escolha | Versão em 2026-11-17 |
| --- | --- | --- |
| Pacotes | Bun instala e trava o `bun.lock` | 1.4.4 |
| Runtime das ferramentas | Node roda o Vite e o SvelteKit | 26.13.1 |
| Framework | SvelteKit · Svelte | 3.3.1 · 5.57.3 |
| Adaptador | `@sveltejs/adapter-static` | 4.0.1 |
| Tipos | TypeScript · `svelte-check` como gate | 6.0.3 · 4.7.8 |
| Estilo | CSS nativo no `<style>` de cada componente, com escopo; os tokens da marca como variáveis CSS; sem Tailwind e sem CSS Modules | — |
| Interação | `bits-ui`, sem estilo, para o acordeão e o menu do celular | 2.21.0 |
| Animação | transições e `motion` do próprio Svelte e CSS, atrás de `prefers-reduced-motion` | — |
| Imagem | `@sveltejs/enhanced-img`, otimizada no build | 1.0.1 |
| Fonte | `@fontsource-variable/atkinson-hyperlegible-next`, o mesmo pacote do web | 5.3.0 |
| Lint e formato | ESLint com `eslint-plugin-svelte` e typescript-eslint type-aware · Prettier com `prettier-plugin-svelte` | 10.15.0 · 3.24.1 · 8.73.0 · 3.10.1 · 4.2.0 |
| Testes | Vitest com jsdom e `@testing-library/svelte` | 5.1.0 · 31.0.1 · 5.4.2 |
| SEO | `seo.svelte` por `<svelte:head>`, `sitemap.xml` prerenderizado por `+server.ts`, `robots.txt` estático, JSON-LD no layout e `og.png` desenhado | — |

O gate do build confere que o `build/index.html` prerenderizado tem o conteúdo da landing.

No scaffold (#5), o Svelte passou para a 5.58.0, a última. O TypeScript ficou no 6.0.3, porque o
SvelteKit 3.3.1 aceita só o `^6.0.0`. O SvelteKit 3 recebe a config no plugin do `vite.config.ts`, sem
`svelte.config.js`, e troca o `$lib` pelo `#lib` do `imports` do `package.json`. Para o
`eslint-plugin-boundaries` resolver esse `#lib`, entrou o `eslint-import-resolver-typescript` 4.4.5. O
`bits-ui`, o `enhanced-img`, o JSON-LD e o `og.png` entram com a primeira seção que precisar deles.

## Consequences

- **O site deixa de ter servidor em produção.** A imagem do Dockerfile passa a ser nginx com os arquivos
  do `build/`, e o Node só existe no build.
- **A frase "o `apps/site` tem servidor, mas não tem sessão" do `AGENTS.md` deixa de valer:** ele não tem
  servidor. Se um dia chamar a API, continua sendo por rota pública com a origem dele em
  `ALLOWED_ORIGINS`.
- **SEO não perde nada com o estático.** O HTML de cada página sai pronto do build, que é o que o SSR
  entregaria, sem servidor para manter.
- **Vídeo só entra quando existir o primeiro.** Ele é codificado uma vez com ffmpeg e os arquivos gerados
  são commitados; o procedimento vai numa skill do projeto.
- **Motion ou GSAP só entram se uma seção desenhada pedir.**

## Alternatives considered

- **Manter o Next standalone (0008 e 0019)** · rejeitada pelo dono do produto.
- **SvelteKit com SSR no `adapter-node`** · rejeitada por enquanto: a landing não tem conteúdo que mude
  por requisição, e o estático entrega o mesmo HTML ao buscador.
- **Tailwind ou CSS Modules** · rejeitados pelo dono do produto, em favor do `<style>` com escopo que o
  próprio Svelte dá.
- **oxlint** · rejeitado: não lê arquivo `.svelte`.

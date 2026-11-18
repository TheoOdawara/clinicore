# `apps/site` — forma e regras

Aditivo ao `AGENTS.md` da raiz. Stack na ADR 0021, forma na 0026. É a landing pública em
`clinicore.com.br`: SvelteKit com `adapter-static`, tudo prerenderizado e servido pelo nginx. Não tem
servidor, sessão nem segredo.

## Estrutura

```
apps/site/
├── vite.config.ts           a config do SvelteKit (adaptador) e do Vitest; não há svelte.config.js
├── eslint.config.ts         com as fronteiras abaixo, provadas pelo eslint-boundaries.test.ts
├── prerender.test.ts        confere o conteúdo do build/ depois do build
├── fixtures/boundaries/     os caminhos falsos que o teste das fronteiras usa
├── e2e/                     Playwright com axe, sobre o build servido pelo vite preview
├── static/                  só o que precisa do nome exato: robots.txt, favicon.ico
└── src/
    ├── app.html  app.d.ts  error.html
    ├── routes/
    │   ├── +layout.svelte   <svelte:head> comum, JSON-LD, fonte e estilo global
    │   ├── +layout.ts       export const prerender = true
    │   ├── +page.svelte     a landing: só compõe as seções, na ordem
    │   └── sitemap.xml/+server.ts
    └── lib/
        ├── sections/<seção>/    <seção>.svelte, <seção>.spec.ts e o conteúdo dela num .ts tipado
        ├── components/<nome>/   UI genérica de mais de uma seção; interação por bits-ui
        ├── assets/              imagem, vídeo e og.png, sempre importados
        ├── content/             texto e dado que mais de uma seção usa
        └── styles/              global.css é manifesto: tokens.css, reset.css, typography.css
```

## Fronteiras

Conferidas pelo `eslint-plugin-boundaries`:

| Quem          | Importa só                                                      |
| ------------- | --------------------------------------------------------------- |
| `routes/`     | `lib/`                                                          |
| `sections/`   | `components`, `assets`, `content` e `styles`; nunca outra seção |
| `components/` | `components`                                                    |
| `content/`    | `assets`                                                        |

## Padrões

- **Runes do Svelte 5** (`$props`, `$state`, `$derived`); sem store legada e sem `export let`.
- **O `lib` entra por `#lib/...`**, o `imports` do `package.json`, sempre com a extensão (`.svelte`,
  `.ts`). O SvelteKit 3 não tem mais o `$lib`.
- **CSS só no `<style>` do componente, sobre as variáveis dos tokens**; sem Tailwind e sem valor de cor ou
  espaço solto.
- **O `.svelte` da seção só renderiza**; o texto e os dados ficam no `.ts` ao lado, tipados.
- **Tudo é prerenderizado.** O texto que o buscador precisa está no HTML do build; o gate confere o
  conteúdo do `build/index.html`, no `prerender.test.ts`.
- **Toda rota tem teste de unidade ao lado** (`page.spec.ts`, `layout.spec.ts`, `server.spec.ts`), sem o
  `+`, que o roteador ignora.
- `shortcut: o site não tem favicon nem og.png porque a marca ainda não tem logo
(docs/product/design/tokens.md); os dois entram com o logo.`
- **Imagem pelo `@sveltejs/enhanced-img`, importada de `lib/assets/`**, para o Vite gerar hash e
  otimizar. O `static/` fica com o mínimo.
- **Animação por transição do Svelte ou CSS, atrás de `prefers-reduced-motion`.**
- **Sem CMS e sem banco**: o conteúdo muda por commit. O site não lê variável de ambiente; se um dia ler,
  o único leitor é o `src/lib/config.ts`, validado por Zod.
- **Se o site chamar a API**, é por rota pública, com a origem dele em `ALLOWED_ORIGINS`. Redirecionamento
  de autenticação nunca aponta para ele.

## Nomes

Arquivo e pasta em kebab-case, inclusive o `.svelte`. Teste ao lado, em `.spec.ts`. **A URL é pt-BR, sem
acento e sem cedilha** (`/precos`, `/para-clinicas`), porque é conteúdo indexável: as pastas de
`src/routes/` são o único lugar do repo com nome em português.

## Caminhos críticos

Nenhum: o site não tem sessão, dado clínico nem segredo.

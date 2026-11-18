---
name: site-section
description: O passo a passo de uma seção nova ou de uma página nova no apps/site do Clinicore, em SvelteKit estático. Use ao criar ou mudar seção, página, imagem ou SEO no apps/site.
---

# site-section

A forma está no `apps/site/AGENTS.md`.

## Seção nova

1. Criar `src/lib/sections/<seção>/` com `<seção>.svelte`, `<seção>.spec.ts` e o conteúdo num `.ts`
   tipado ao lado (o texto, os itens, os links).
2. O `.svelte` recebe o conteúdo por import e só renderiza; o estilo fica no `<style>` dele, sobre as
   variáveis de `lib/styles/tokens.css`.
3. Imagem entra por `<enhanced:img>` com import de `lib/assets/`.
4. Interação (acordeão, menu) usa o componente de `lib/components/`, construído sobre o `bits-ui`.
5. Animação, quando houver, respeita `prefers-reduced-motion`.
6. A seção entra no `+page.svelte`, na ordem da landing.

## Página nova

1. A pasta em `src/routes/` é a URL, em pt-BR sem acento (`/precos`).
2. A página entra no `sitemap.xml/+server.ts`.
3. O `seo` de `lib/components/` dá o `<title>`, a descrição e o Open Graph dela.

## Testes

- Seção: Vitest com `@testing-library/svelte`, conferindo o texto e os links que o conteúdo define.
- O gate do build confere o conteúdo do `build/index.html` prerenderizado.

# `apps/site`

A landing pública e indexável, em `clinicore.com.br`: SvelteKit com `adapter-static`, tudo
prerenderizado e servido pelo nginx ([ADR 0021](../../../decisions/0021-site-em-sveltekit-estatico.md)).

**Nunca faz:** participar de sessão, ler cookie, receber segredo, chamar rota autenticada ou ser destino
de redirecionamento de autenticação. Não tem servidor, CMS nem banco: o conteúdo mora no repo.

O scaffold (#5) publica só o nome do produto, com o `<title>`, a descrição, o `sitemap.xml` e o
`robots.txt`. As seções da landing são do M3.

## Blocos internos

| Bloco | Onde | Responsabilidade | Estado |
| --- | --- | --- | --- |
| Rotas | `src/routes/` | uma pasta por URL em pt-BR, o layout, o `sitemap.xml` | o layout, a página inicial e o `sitemap.xml` existem |
| Seções | `src/lib/sections/` | os blocos da landing: hero, preços, dúvidas, com o conteúdo tipado ao lado | M3 · `FR-SITE-01` a `FR-SITE-04` |
| Componentes | `src/lib/components/` | a UI base da landing, com a interação sobre o `bits-ui` | o `seo` existe; o resto é M3 |
| Mídia | `src/lib/assets/` | imagem e vídeo, otimizados no build | M3 |
| Estilo | `src/lib/styles/` | os tokens de `docs/product/design/` como variáveis CSS | existe |

## Para mudar com segurança

As fronteiras, os nomes e a regra de URL em pt-BR estão no
[`apps/site/AGENTS.md`](https://github.com/TheoOdawara/clinicore/blob/staging/apps/site/AGENTS.md). O gate do site confere o conteúdo do HTML
prerenderizado, porque build verde não garante página com conteúdo.

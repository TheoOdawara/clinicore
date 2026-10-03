# `apps/site`

A landing pública e indexável, em `clinicore.com.br`: Next.js 16 com App Router, entregue como servidor
standalone ([ADR 0008](../../decisions/0008-site-em-next-standalone.md)).

**Nunca faz:** participar de sessão, ler cookie, receber segredo, chamar rota autenticada ou ser destino
de redirecionamento de autenticação. Não tem CMS nem banco: o conteúdo mora no repo.

## Blocos internos

| Bloco | Onde | Responsabilidade | Estado |
| --- | --- | --- | --- |
| Páginas | `src/app/` | uma pasta por URL, com `layout.tsx` e `page.tsx` | existe, com a página inicial |
| Estilo | `src/styles/` | Tailwind e os tokens | existe |
| Seções | `src/sections/` | os blocos da landing: hero, preços, dúvidas | M3 · `FR-SITE-01` a `FR-SITE-04` |
| Componentes | `src/components/` | a UI base do shadcn/ui, em cópia própria | M3 |
| Mídia | `src/assets/` | imagem e vídeo da landing | M3 |

## Para mudar com segurança

As regras de Server Component, imagem, fonte e URL em pt-BR estão no
[`apps/site/AGENTS.md`](../../../apps/site/AGENTS.md). O gate do site confere o conteúdo do HTML
pré-renderizado, porque build verde não garante página com conteúdo.

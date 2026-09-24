# Clinicore `apps/site` — páginas públicas e pegadinhas

Aditivo ao `CLAUDE.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este
arquivo apenas onde ela falar do mesmo assunto. A raiz tem a stack, os comandos, a visão geral da
arquitetura, as branches e o idioma.

## Páginas públicas

Next.js com App Router, entregue como servidor standalone. A ADR 0008 trocou o Vite da 0002 por
isto: `next/image` otimizando a mídia da landing, e rota dinâmica sem trocar o deploy.

- **Uma pasta por página em `src/app/`**, com a página composta a partir de `sections/`. O conteúdo
  da landing mora na seção, não na rota.
- **Página é Server Component por padrão.** `"use client"` entra só no componente que precisa de
  estado, efeito ou evento, e o mais fundo possível na árvore — nunca na página nem no layout.
- **O texto que precisa ser indexado é renderizado no servidor.** Componente cliente que só mostra
  conteúdo depois de montar deixa o HTML do build sem esse conteúdo, com o build verde.
- **Imagem entra por `next/image` com import estático**, que dá `width` e `height` sozinho e mantém o
  `CLS = 0` do contrato global. Imagem acima da dobra leva `preload`; o `priority` está obsoleto no
  16.3 e o `no-deprecated` reprova.
- **Fonte entra por `next/font`**, servida do próprio domínio.
- **Vídeo não passa por build nenhum.** Encode, poster e `preload` são decisão de quem escreve a seção.
- **Aqui não há restrição de categoria do React Bits**, e animação de rolagem que atravessa várias
  seções é escrita com o GSAP direto no DOM, num efeito de um único componente cliente, não repartida
  entre componentes.
- **Sem CMS e sem banco.** O conteúdo mora no repo e muda por commit.
- **Nada de sessão nem de segredo.** O servidor do site não lê cookie, não chama rota autenticada e
  nunca é destino de redirecionamento de autenticação. O código do app não lê variável de ambiente; o
  `PORT` e o `HOSTNAME` do container são lidos pelo `server.js` do standalone, não por nós.
- **URL é pt-BR, sem acento e sem cedilha**, porque aqui o caminho é conteúdo indexável e é o que a
  pessoa lê antes de clicar: `/precos`, `/funcionalidades`, `/para-clinicas`. Como o nome da pasta em
  `src/app/` é a URL, este é o único lugar do repo onde nome de pasta não é inglês — componente, prop,
  variável e os arquivos de convenção do Next (`page.tsx`, `layout.tsx`) continuam sendo.

## Pegadinhas da stack

Verificadas em 2026-09-24, contra `next@16.3.6`.

- **Build verde não garante página com conteúdo.** O `next build` falha quando uma página lança erro no
  prerender, mas não quando um componente cliente renderiza vazio no servidor. Por isso o gate do site
  confere o texto dentro de `.next/server/app/index.html`, e não só que o build passou.
- **O standalone não copia os arquivos estáticos.** `.next/standalone/` sai sem `.next/static/` e sem
  `public/`; quem monta a imagem de container copia os dois, senão a página sobe sem CSS e sem
  JavaScript.
- **O `sharp` vem como dependência opcional do `next`** e é empacotado no standalone. Um `npm ci` com
  `--omit=optional` tira ele, e o `next/image` deixa de otimizar.
- **O `eslint-config-next` não substitui o `typescript-eslint`.** Os dois entram juntos, para manter o
  `@typescript-eslint/no-deprecated` que a ADR 0004 exige nos três apps.

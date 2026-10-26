# Clinicore `apps/site` — páginas públicas e pegadinhas

Aditivo ao `AGENTS.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este
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
  `CLS = 0` do contrato global. Imagem acima da dobra leva `preload`, nunca `priority`, que está
  obsoleto no 16.3.
- **Fonte entra por `next/font`**, servida do próprio domínio.
- **Vídeo não passa por build nenhum.** Encode, poster e `preload` são decisão de quem escreve a seção.
- **Animação é CSS nativo e, quando não basta, Motion**, que entra com a primeira seção animada. GSAP e
  three.js só entram se uma seção desenhada pedir uma timeline fixada ou uma cena 3D, cada um com a sua
  linha aqui. **Aqui não há restrição de categoria do React Bits**; o componente copiado traz a dependência
  dele, e a variante em Motion é a preferida.
- **Sem CMS e sem banco.** O conteúdo mora no repo e muda por commit.
- **Nada de sessão nem de segredo.** O servidor do site não lê cookie, não chama rota autenticada e
  nunca é destino de redirecionamento de autenticação. O código do app não lê variável de ambiente; o
  `PORT` e o `HOSTNAME` do container são lidos pelo `server.js` do standalone, não por nós.
- **URL é pt-BR, sem acento e sem cedilha**, porque aqui o caminho é conteúdo indexável e é o que a
  pessoa lê antes de clicar: `/precos`, `/funcionalidades`, `/para-clinicas`. Como o nome da pasta em
  `src/app/` é a URL, este é o único lugar do repo onde nome de pasta não é inglês — componente, prop,
  variável e os arquivos de convenção do Next (`page.tsx`, `layout.tsx`) continuam sendo.

## Runtime e testes

- **O Node 26 roda o Next; o Bun só instala e roda as ferramentas.** `bun run dev` e `bun run build`
  sobem o `next` pelo shebang dele, que é Node. O oxlint, o `bun fmt`, o `bun check` e o Vitest são
  ferramentas, e o servidor de produção é o `server.js` do standalone em `node:26.12.1-slim`.
- **A cobertura é 100% sobre `src/` fora de `src/app/`.** Página e layout são Server Components, que o
  Vitest não renderiza quando são `async`; quem garante o conteúdo deles é o gate de prerender. Componente
  cliente e lógica em `components/` e `sections/` têm teste em `__tests__/` ao lado.

## Pegadinhas da stack

Verificadas em 2026-09-24 contra `next@16.3.6` e reconfirmadas em 2026-10-25 contra `next@16.4.3`.

- **Build verde não garante página com conteúdo.** O `next build` falha quando uma página lança erro no
  prerender, mas não quando um componente cliente renderiza vazio no servidor. Por isso o gate do site
  confere o texto dentro de `.next/server/app/index.html`, e não só que o build passou.
- **O standalone não copia os arquivos estáticos.** `.next/standalone/` sai sem `.next/static/` e sem
  `public/`; quem monta a imagem de container copia os dois, senão a página sobe sem CSS e sem
  JavaScript.
- **O `sharp` vem como dependência opcional do `next`** e é empacotado no standalone. Um `bun install`
  com `--omit=optional` tira ele, e o `next/image` deixa de otimizar.
- **O `typescript/no-deprecated` do oxlint não pega prop obsoleta em JSX.** Medido com o oxlint 1.89.0:
  `"abc".substr(1)` reprova, e `<Image priority />` passa verde. Prop obsoleta de componente é conferida no
  review.
- **Nenhum script do `package.json` se chama `check` nem `fmt`.** O `bun check` e o `bun fmt` rodam o
  script de mesmo nome quando ele existe, e o gate deixaria de ser o type checker e o formatador.
- **O `bun check` só vê os tipos de rota depois de um `next build`**, porque o `next-env.d.ts` e o
  `.next/types/` são gerados por ele e ficam fora do git.
- **O `next dev` escreve regras de agente no `CLAUDE.md` e no `AGENTS.md` do app** a cada subida, em inglês e dentro
  de um comentário HTML. O `agentRules: false` do `next.config.ts` desliga isso e não sai.

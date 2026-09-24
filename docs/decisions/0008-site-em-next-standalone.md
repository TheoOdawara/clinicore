# 0008. Páginas públicas em Next.js, com servidor standalone

- Status: accepted
- Date: 2026-09-24
- Emenda a: 0002 (a stack e a entrega do `apps/site`), issue #4 (o site passa a ter processo próprio no
  deploy), `docs/specs/002-scaffold-apps-e-ci.md` (task 5) e `CLAUDE.md` da raiz (Stack, Comandos e
  Arquitetura do `apps/site`)

## Context

A ADR 0002 criou o `apps/site` na mesma stack do `apps/web` — Vite, React e TanStack Router —, com o
HTML de indexação gerado no build pelo `vite-prerender-plugin` e a imagem comprimida pelo
`vite-imagetools`. O motivo declarado era um só: se um dia landing e sistema fossem fundidos, a fusão
seria mover pasta e não reescrever.

O `apps/site` ainda não tem uma linha de código. A issue #76, que o cria, não começou.

O dono do produto decidiu que as páginas públicas do Clinicore são escritas em Next.js, entregue como
servidor Node standalone. Dois ganhos sustentam a escolha, e nenhum deles alcança o `apps/web`:

1. **A landing é pesada de mídia**, e o `next/image` otimiza sob demanda: redimensiona, converte para
   AVIF e WebP e escreve `width` e `height` a partir do import estático. O `vite-imagetools` fazia a
   conversão no build, mas deixava as dimensões e o `CLS = 0` para serem escritos à mão em cada
   componente.
2. **Rota dinâmica e ISR ficam disponíveis sem trocar o modelo de deploy.** Sitemap gerado, imagem de
   Open Graph por página e conteúdo que muda sem rebuild passam a ser uma rota nova, não uma migração de
   estático para servidor.

**A indexação não é ganho do servidor, e esta ADR não a usa como motivo.** Medido numa build do
`next@16.3.6` com `output: "standalone"`: a rota `/` sai marcada como estática e prerenderizada no build,
em `.next/server/app/index.html`, com o texto da página dentro. O crawler recebe o mesmo HTML que o
prerender da 0002 entregaria.

## Decision

**O `apps/site` nasce em Next.js 16.3.6, com App Router e `output: "standalone"`.** O `apps/web`
continua em Vite e TanStack Router: nada da 0002 sobre ele muda.

| | `apps/site` |
| --- | --- |
| Framework | Next.js 16.3.6, App Router, `output: "standalone"` |
| UI | React 19.3 |
| Tipos | TypeScript 6.0 |
| Lint e formato | ESLint 9 · `typescript-eslint` 8 com `strictTypeChecked` e `stylisticTypeChecked` · `eslint-config-next` 16.3.6 · Prettier 3 |
| Estilo | Tailwind 4.3, pelo `@tailwindcss/postcss` · shadcn/ui e React Bits por cópia própria, pela 0003 |
| Imagem e fonte | `next/image` com o `sharp` que o Next traz como dependência opcional · `next/font` |
| Pacotes | npm sobre Node 26, pela 0005 |
| Porta de desenvolvimento | :4321 |
| Processo em produção | `node server.js`, de `.next/standalone/` |

Decisões de detalhe dentro desta:

- **Saem do site o TanStack Router, o `vite-prerender-plugin` e o `vite-imagetools`.** O
  `src/prerender.tsx` deixa de existir, e com ele a regra de nunca tocar `window` no caminho da
  `prerender()`.
- **A estrutura passa a ser `src/app/` do App Router**, no lugar de `src/routes/` e `src/main.tsx`.
  `src/sections/`, `src/components/`, `src/assets/` e `src/styles/` continuam com o mesmo papel.
- **A URL continua pt-BR, sem acento e sem cedilha**, e o nome da pasta em `src/app/` é a URL:
  `src/app/precos/page.tsx` serve `/precos`. Continua sendo o único lugar do repo onde nome de pasta
  não é inglês.
- **O `eslint-config-next` entra ao lado do `typescript-eslint`, não no lugar dele.** O
  `@typescript-eslint/no-deprecated` que a 0004 exigiu nos três apps continua ligado como `error`. O
  `eslint-config-next@16.3.6` declara peer `eslint >=9.0.0` e `typescript >=3.3.1`, e já traz o
  `eslint-plugin-jsx-a11y` e o `eslint-plugin-react-hooks` que a 0004 instalou no web.
- **O site continua sem sessão, sem segredo e sem variável de aplicação.** Ter servidor não muda
  isso: ele não lê cookie, não chama rota autenticada e nunca é destino de redirecionamento de
  autenticação. O container recebe só o `PORT` e o `HOSTNAME` que o `server.js` do standalone lê, e
  eles são configuração do processo, não do app. Quando chamar a API, é por rota pública, do browser, e a origem dele entra em
  `ALLOWED_ORIGINS`.
- **O gate do HTML continua no CI**, agora sobre `.next/server/app/index.html`. O build do Next falha
  quando uma página lança erro no prerender, que era o buraco que a 0002 cobria; o que sobra é um
  componente cliente que só renderiza depois de montar, e ele gera HTML sem o conteúdo com o build
  verde. O passo confere que o texto `Clinicore` está no arquivo, fora de atributo.
- **O site ganha processo próprio no deploy.** O `compose.yaml` da raiz, da #4, passa a ter seis
  serviços, com o `site` rodando o `server.js` do standalone. A #4 é emendada junto com esta ADR.

## Consequences

Fica mais fácil:

- **Imagem sem trabalho manual.** O import estático dá `width` e `height` ao `next/image`, e o
  `CLS = 0` do contrato global deixa de depender de cada componente lembrar das dimensões.
- **Página dinâmica é rota nova, não migração.** O servidor já existe no deploy.
- **O prerender deixa de quebrar em silêncio por exceção.** O build do Next falha quando uma página
  estática lança erro.
- **Fonte sem layout shift**, com o `next/font` servindo do próprio domínio.

Fica mais difícil, e é aceito:

- **A fusão landing ↔ sistema deixa de ser mover pasta.** Era o único motivo da stack única na 0002.
  Fundir os dois passa a ser reescrever as páginas públicas no router do web, ou o web no do site.
- **O front passa a ter duas stacks.** Router, convenção de arquivo, carregamento de imagem e config de
  build diferem entre `apps/web` e `apps/site`. O que continua igual é React, TypeScript, lint, formato,
  Tailwind e o gerenciador de pacotes.
- **Mais um runtime em produção.** O site passa a ser um processo Node para rodar, monitorar, atualizar
  e proteger, exposto à internet. A superfície fica pequena porque ele não tem segredo, sessão nem dado
  de paciente, mas as atualizações de segurança do Next passam a ser trabalho recorrente.
- **O standalone não copia os arquivos estáticos.** Medido: `.next/standalone/` sai sem `.next/static/`
  e sem `public/`. A imagem de container do site copia os dois para dentro do standalone, senão a
  página carrega sem CSS e sem JavaScript.
- **O cache de imagem otimizada mora no disco do container do site**, em `.next/cache/images`. Um
  container novo refaz as variantes na primeira visita. São imagens públicas da landing, não dado de
  saúde, e a restrição da 0002 contra o `next/image` continua valendo para foto de paciente, que nunca
  passa por este app.
- **O CI do site fica mais lento**, porque o `next build` roda o TypeScript e gera as páginas.

Gatilho que reabre esta decisão: **o site precisar de sessão ou de dado autenticado.** Nesse ponto o
modelo de ameaça muda, porque é um segundo servidor perto do usuário logado, e a decisão volta a ser
tomada com a 0002 aberta ao lado.

## Alternatives considered

- **Manter o `apps/site` em Vite, como a 0002 decidiu** · rejeitada pelo dono do produto. Mantinha a
  fusão como mover pasta e evitava o processo no deploy, mas obrigava a escrever as dimensões de imagem
  à mão e empurrava qualquer página dinâmica para uma migração futura.
- **Next.js com `output: "export"`** · rejeitada pelo dono do produto. Não tinha processo no deploy e a
  indexação era a mesma, mas desligava a otimização do `next/image`, que é metade do motivo da troca.
  Obrigava `images.unoptimized: true` e deixava a conversão para AVIF e WebP de fora.
- **TanStack Start no `apps/site`** · rejeitada: dava servidor mantendo o router do web, que é o que a
  0002 nomeou como saída para renderização no servidor. A escolha do dono do produto foi o ecossistema
  do Next, com o otimizador de imagem que vem nele.
- **Next.js também no `apps/web`, para voltar a ter uma stack** · rejeitada: as razões da 0002 contra o
  Next na área logada continuam de pé. Os cookies da API não alcançam um servidor do web sem proxy, e
  dado de saúde passaria a tocar um segundo runtime.

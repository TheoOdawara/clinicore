# Clinicore `apps/site` — estático e pegadinhas

Aditivo ao `CLAUDE.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este
arquivo apenas onde ela falar do mesmo assunto. A raiz tem a stack, os comandos, a visão geral da
arquitetura, as branches e o idioma.

## Estático

Mesma stack do `web`, outro produto. A ADR 0002 escolheu assim para que fundir os dois, se isso for
decidido, seja mover pasta e não reescrever.

- **Uma página por arquivo em `routes/`**, composta a partir de `sections/`. O conteúdo da landing mora
  na seção, não na rota.
- **O HTML da indexação sai do build, não do servidor.** `src/prerender.tsx` exporta a função
  `prerender()` que o `vite-prerender-plugin` chama: ela monta o router com `createMemoryHistory`,
  renderiza para string e devolve `{ html, links, head }`. **Nada nesse caminho pode tocar `window`,
  `document` ou `localStorage` fora de efeito** — se tocar, o build sai verde com HTML vazio e a página
  deixa de ser indexável sem avisar.
- **Imagem entra por `vite-imagetools`**, com o tamanho e o formato pedidos no import, e o componente
  escreve `width` e `height` à mão, porque o plugin não escreve. Sem isso o `CLS = 0` do contrato
  global cai.
- **Vídeo não passa por build nenhum.** Encode, poster e `preload` são decisão de quem escreve a seção.
- **Aqui não há restrição de categoria do React Bits**, e animação de rolagem que atravessa várias
  seções é escrita com o GSAP direto no DOM, num efeito da rota, não repartida entre componentes.
- **Sem CMS e sem banco.** O conteúdo mora no repo e muda por commit.
- **Nada de sessão.** O site não lê cookie, não chama rota autenticada e nunca é destino de
  redirecionamento de autenticação.
- **URL é pt-BR, sem acento e sem cedilha**, porque aqui o caminho é conteúdo indexável e é o que a
  pessoa lê antes de clicar: `/precos`, `/funcionalidades`, `/para-clinicas`. Como o nome do arquivo
  em `routes/` é a URL, este é o único lugar do repo onde nome de arquivo não é inglês — componente,
  prop e variável do site continuam sendo.

## Pegadinhas da stack

Verificadas em 2026-09-15, contra as versões desta stack.

- **Build verde não significa página indexável.** O `vite-prerender-plugin` chama a `prerender()` do
  app; se ela lançar, o que sobra é o `index.html` do Vite, que é uma `<div>` vazia. O `tsc` não pega,
  o Biome não pega e o build sai com código 0. Por isso o gate do site confere o conteúdo do HTML
  gerado, e não só que o build passou.
- **`window` fora de efeito é o jeito mais comum de derrubar o prerender**, porque a `prerender()` roda
  em Node. Vale para código copiado do React Bits, que costuma ler `window` na montagem.
- **O `vite-imagetools` não escreve `width` nem `height`.** Ele entrega o arquivo otimizado e mais nada;
  a dimensão no HTML é trabalho do componente.

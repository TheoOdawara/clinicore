# Design

A marca e as telas do Clinicore são desenhadas no **Claude Design**, o canvas de design do claude.ai,
com o plugin **ui-ux-pro-max** do Claude Code para a direção visual: paleta, tipografia, tokens e a
revisão de usabilidade de cada tela.

O canvas não exporta imagem por ferramenta. A imagem de cada tela sai do HTML estático do artboard,
renderizado no Chromium headless e convertido para WebP, e fica ao lado da página que a documenta.

## O que existe

A marca ainda não foi definida, e nenhuma tela foi desenhada. Por isso esta pasta ainda não tem:

- `brand/`, com a descrição da marca, o brand book, o logo e o banner, que nascem com a marca
- `tokens.md`, com os tokens de cor, tipo, espaço, raio e sombra em claro e escuro; os mesmos valores
  entram no `docs/stylesheets/docs-theme.css` do site local
- `pages/<feature>/<página>.md`, uma por tela, que nascem com a spec de cada feature: a imagem em
  390px e depois no desktop, as áreas, os estados e os tokens que ela usa

A base de componentes do `apps/web` é o shadcn/ui, pela
[ADR 0003](../decisions/0003-design-system-com-shadcn-ui-e-react-bits.md), adotado na #77.

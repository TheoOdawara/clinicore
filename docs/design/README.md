# Design

A marca e as telas do Clinicore são desenhadas em HTML e CSS estáticos, versionados neste repositório,
com o plugin **ui-ux-pro-max** do Claude Code para a direção visual: paleta, tipografia, tokens e a
revisão de usabilidade de cada tela. O repositório é a única origem: nenhum design mora fora dele.

A imagem de cada tela sai desse HTML, renderizado no Chromium headless e convertido para WebP, e fica ao
lado da página que a documenta.

## O que existe

- [`tokens.md`](tokens.md), com os valores em [`tokens.css`](tokens.css): cor em claro e escuro, tipo,
  raio e espaço. Os mesmos valores de cor entram no `docs/stylesheets/docs-theme.css` do site local.
- `pages/auth/`, com as telas de [entrar](pages/auth/login.md), [criar conta](pages/auth/signup.md) e
  [confirmar o e-mail](pages/auth/verify-email.md).

Cada tela é `pages/<feature>/<página>.html`, com `pages/prototype.css` e `pages/prototype.js`. Para
gerar as imagens de novo, abra o HTML no Chrome headless em `--window-size=<largura>,1000` e converta com
`cwebp`. Com `?dark`, o HTML mostra o tema escuro, e com `?desktop` mostra só o primeiro estado, na tela
inteira.

`brand/` tem os estudos da marca. A descrição da marca, o brand book, o logo e o banner nascem quando o
produto ganhar nome, porque "Clinicore" é codinome.

Os tokens alimentam o tema de cada app: o `mat.theme` do Angular Material no web (ADR 0020), as variáveis
CSS do site (ADR 0021), o tema Material 3 do Android e o `DesignSystem` do iOS (ADR 0022).

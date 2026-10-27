# Design

A marca e as telas do Clinicore são desenhadas no **Claude Design**, o canvas de design do claude.ai,
com o plugin **ui-ux-pro-max** do Claude Code para a direção visual: paleta, tipografia, tokens e a
revisão de usabilidade de cada tela.

O canvas não exporta imagem por ferramenta. A imagem de cada tela sai do HTML estático do artboard,
renderizado no Chromium headless e convertido para WebP, e fica ao lado da página que a documenta.

## O que existe

- [`tokens.md`](tokens.md), com os valores em [`tokens.css`](tokens.css): cor em claro e escuro, tipo,
  raio e espaço. Os mesmos valores de cor entram no `docs/stylesheets/docs-theme.css` do site local.
- `pages/auth/`, com as telas de [entrar](pages/auth/login.md), [criar conta](pages/auth/signup.md) e
  [confirmar o e-mail](pages/auth/verify-email.md).

**Esses protótipos nasceram fora do canvas**, em HTML estático no próprio repositório
(`pages/<feature>/<página>.html`, com `pages/prototype.css` e `pages/prototype.js`). A conta do Claude
Design não estava disponível. Eles serão levados para o canvas quando ela estiver, e a partir daí o canvas
volta a ser a origem. Para gerar as imagens de novo, abra o HTML no Chrome headless em
`--window-size=<largura>,1000` e converta com `cwebp`. Com `?dark`, o HTML mostra o tema escuro, e com
`?desktop` mostra só o primeiro estado, na tela inteira.

Ainda não existe `brand/`, com a descrição da marca, o brand book, o logo e o banner. Ela nasce quando o
produto ganhar nome, porque "Clinicore" é codinome.

O estilo do `apps/web` e do `apps/site` é Tailwind 4.3, com o shadcn/ui como design system (ADR 0003).

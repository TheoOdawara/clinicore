---
name: design
description: Como a marca e as telas do Clinicore são desenhadas e documentadas, só em HTML versionado no repositório. Use ao desenhar marca, tela ou componente, ao mudar tokens, e ao gerar a imagem de uma tela.
---

# design

- **Todo design mora no repositório, em HTML e CSS estáticos.** Nenhuma ferramenta externa é origem de
  design: nem canvas, nem arquivo fora do git.
- **A direção visual é do plugin `ui-ux-pro-max`** do Claude Code: paleta, tipografia, tokens e a
  revisão de usabilidade.
- **Cada tela é `docs/design/pages/<feature>/<página>.html`**, sobre `pages/prototype.css` e
  `pages/prototype.js`, com um `<página>.md` ao lado que documenta os estados. Cada estado é uma
  `<figure class="artboard" data-label="...">`.
- **A imagem sai do HTML no Chromium headless** em `--window-size=<largura>,1000`, convertida para WebP
  com `cwebp` e salva ao lado da página. `?dark` mostra o tema escuro e `?desktop` só o primeiro estado.
- **Os tokens (`docs/design/tokens.css` e `tokens.md`) são a fonte de cor, tipo, raio e espaço de todos
  os apps**: o `mat.theme` do web, as variáveis CSS do site, o tema Material 3 do Android e o
  `DesignSystem` do iOS. Valor solto num componente é proibido.
- **A marca vem antes do nome do produto**, e nunca depende dele: o nome ainda está em aberto.

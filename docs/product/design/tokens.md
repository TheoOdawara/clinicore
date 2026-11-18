# Tokens

Os valores vivem em [`tokens.css`](tokens.css), com os nomes de variável do shadcn/ui, para que o
`apps/web` copie o arquivo sem traduzir nome nenhum. Os protótipos de `pages/` leem o mesmo arquivo.

A direção saiu do `ui-ux-pro-max`, na busca por clínica médica e odontológica: estilo minimalista e
funcional, azul-petróleo como cor de ação e uma família tipográfica feita para leitura. Os valores
sugeridos foram ajustados para passar no contraste da WCAG AA, como diz cada linha abaixo.

São provisórios até a marca existir. "Clinicore" é codinome
([dúvidas em aberto](../requirements/open-questions.md)), e por isso não há logo: o nome aparece como
texto, na cor primária.

## Cor

| Token | Claro | Escuro | Por quê |
| --- | --- | --- | --- |
| `--background` | `#f7fafa` | `#0b1a1f` | fundo quase neutro, para uma tela que fica aberta o dia inteiro na recepção |
| `--foreground` | `#0f2a33` | `#e6f1f3` | texto com 14,3:1 e 15,4:1 sobre o fundo |
| `--card` | `#ffffff` | `#11242a` | a superfície do formulário se destaca do fundo sem sombra pesada |
| `--primary` | `#0e7490` | `#22b8cf` | a paleta sugeria `#0891b2`, que dá só 3,7:1 com texto branco; o tom mais escuro dá 5,4:1 |
| `--primary-foreground` | `#ffffff` | `#04232a` | 5,4:1 no claro e 6,9:1 no escuro |
| `--secondary`, `--accent` | `#e6f0f2` | `#17303a` | hover do botão contornado |
| `--muted` | `#eef4f5` | `#17303a` | — |
| `--muted-foreground` | `#4b5f68` | `#9db4bb` | texto de apoio, com 6,7:1 e 7,4:1 sobre o cartão |
| `--destructive` | `#c81e1e` | `#f87171` | erro, com 5,7:1 e 5,8:1 sobre o cartão |
| `--border` | `#d6e2e5` | `#23404a` | divisória e contorno do cartão, que são decoração |
| `--input` | `#778d95` | `#4e7380` | contorno do campo, com 3,5:1 e 3,1:1, porque a WCAG 1.4.11 pede 3:1 em componente |
| `--ring` | `#0e7490` | `#22b8cf` | anel de foco, igual à primária |

## Tipo

**Atkinson Hyperlegible Next**, uma família só, nos pesos 400, 500, 600 e 700. Ela foi desenhada para
distinguir as letras que se confundem (`I`, `l` e `1`, `O` e `0`), o que conta em e-mail, senha e, mais
adiante, em CPF e número de carteirinha. Ela cobre os acentos do português.

| Uso | Tamanho | Peso |
| --- | --- | --- |
| Título do cartão | 24px | 600 |
| Corpo, campo e botão | 16px | 400, e 600 no botão |
| Rótulo, texto auxiliar, erro de campo | 14px | 600 no rótulo, 400 nos demais |

No `apps/web` a fonte entra pelo `@fontsource-variable`, servida pelo próprio app, para que o PWA
funcione sem conexão.

## Forma e espaço

| Token | Valor | Uso |
| --- | --- | --- |
| `--radius` | `0.5rem` | cartão com `--radius` + 4px, campo e botão com `--radius` − 2px |
| Altura de campo e botão | 44px | o alvo de toque mínimo |
| Margem lateral no celular | 16px | regra 6 da spec do web |
| Largura máxima do cartão | 400px | a partir de 640px |

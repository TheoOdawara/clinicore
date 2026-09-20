# 0003. Design system com shadcn/ui, e React Bits com escopo por categoria

- Status: accepted
- Date: 2026-09-20

## Context

A spec `docs/specs/autenticacao/003-autenticacao-web.md` dizia que as telas usariam "utilitárias do
Tailwind diretamente, sem design system", e listava design system em Fora de Escopo, numa fase que ainda
não tinha issue. Sete telas de autenticação estão prestes a ser escritas, e a regra 8 dessa spec —
`label` associado, `aria-describedby`, `role="alert"` e foco visível — seria escrita à mão em cada uma
delas.

A ADR 0002 fixou dois apps de front na mesma stack e com naturezas opostas: o `apps/web` é ferramenta de
trabalho usada o dia inteiro em computador e tablet da clínica; o `apps/site` é landing pública, estática,
onde efeito visual é parte do produto e o JavaScript já está pago pela mídia da página.

## Decision

**shadcn/ui é o design system do `apps/web`**, sobre Radix, Tailwind 4 e CVA, com os componentes copiados
para `apps/web/src/shared/`. Entram junto o alias `@/`, o `components.json` e o `tw-animate-css` 1.4.0,
que é o que o próprio shadcn usa no Tailwind 4.

**shadcn também serve o `apps/site`**, pelos mesmos comandos e com o mesmo `components.json`, porque os
dois apps são React sobre Vite. **A cópia é feita duas vezes, uma por app.** Os apps são independentes e
não importam código um do outro, então botão, acordeão e diálogo da landing são arquivos do `apps/site`,
não um pacote comum — e divergir entre os dois é permitido, porque a landing e o sistema não precisam
parecer a mesma tela.

**React Bits entra nos dois apps, por cópia (`jsrepo` 3.8.1), com escopo por categoria:**

| Categoria | Exemplos | `apps/site` | `apps/web` |
| --- | --- | --- | --- |
| Fundo animado e WebGL | Aurora, Particles, Waves, Liquid Chrome, Grid Motion | sim | **não** |
| Texto animado | Split Text, Blur Text, Shiny Text, Decrypted Text | sim | **não sobre dado clínico** |
| Contador | Count Up | sim | sim |
| Entrada e transição | Animated Content, Fade Content, Star Border | sim | sim |
| Cursor | Magnet, Click Spark, Splash Cursor | sim | **não** |
| Componente | Stack, Carousel, Dock, Stepper, Masonry, Tilted Card | sim | caso a caso |

Os motivos das três proibições no `apps/web`: fundo animado e WebGL rodam GPU o dia inteiro num tablet
que precisa durar o expediente; efeito de cursor não existe em toque, que é como a recepção opera; e
nome de paciente, valor e resultado não aparecem por animação de texto. Onde o shadcn tiver equivalente,
ele tem prioridade — não se mantém dois sistemas de animação para o mesmo componente.

**Quatro regras valem para todo componente copiado, de qualquer das duas fontes**, e entram como
critério de aceite da issue que o usar:

1. **`prefers-reduced-motion` é respeitado.** Parte dos componentes do React Bits não honra isso de
   fábrica; quem copia, edita. Software de saúde é usado oito horas por dia.
2. **Nenhuma animação segura informação.** Transição de entrada não atrasa o dado na tela.
3. **Antes de copiar, checa-se se uma classe resolve.** O `tw-animate-css` já cobre fade, slide, zoom e
   accordion.
4. **O comentário do arquivo copiado é apagado na mesma edição**, porque o repo só admite o marcador
   `ponytail:`.

Sobre licença, conferido nas versões desta stack: `motion` 13.4.0 é MIT, `ogl` 1.0.11 é Unlicense e o
`jsrepo` 3.8.1 é MIT. O **`gsap` 3.15.0 não é MIT** — é a "Standard no-charge license" própria da GSAP,
gratuita e suficiente para animação de interface num SaaS, mas não-OSI. Ela foi lida e aceita; entre duas
variantes do mesmo componente, prefere-se a que usa `motion`.

## Consequences

Fica mais fácil:

- A regra 8 da spec de autenticação deixa de ser trabalho manual em sete telas: o Radix já entrega rótulo
  associado, descrição por `aria-describedby`, anúncio de erro e foco visível.
- A fase de tema e tipografia passa a ter onde morar, sem reescrever componente.
- A landing tem efeito visual de verdade sem nada a traduzir: o componente é o mesmo `.tsx` dos dois
  lados, e o que for escrito num app roda no outro se um dia forem fundidos.

Fica mais difícil, e é aceito:

- **Todo arquivo que o `shadcn add` ou o `jsrepo` gera vem comentado**, e o contrato do repo só admite
  `ponytail:`. A limpeza é manual e é recorrente, a cada componente adicionado.
- **Os dois apps de front ganham o Radix como dependência real**, por componente usado, cada um no
  seu manifesto.
- **Existem duas fontes de componente no mesmo app**, e a tabela acima é o que impede que virem duas
  bibliotecas concorrentes.
- A spec `003-autenticacao-web.md` é emendada nas duas linhas que diziam "sem design system".

## Alternatives considered

- **Seguir em Tailwind puro nas sete telas** · rejeitada: repetiria a regra 8 sete vezes à mão, e a fase
  de design system reescreveria tudo depois.
- **shadcn só no `apps/web`, com a landing em Tailwind puro** · rejeitada: a landing tem formulário de
  contato, acordeão de dúvidas e diálogo de demonstração, que são exatamente o que o shadcn cobre, e a
  regra 8 da spec de autenticação vale para qualquer formulário público também.
- **React Bits só na landing** · rejeitada: deixaria de fora micro-animação de entrada e o contador, que
  servem aos indicadores das issues #44 e #45 sem nenhum dos custos que motivaram as proibições.
- **React Bits sem tabela de escopo** · rejeitada: sem a tabela, fundo em WebGL e efeito de cursor
  entram no sistema por conveniência de quem implementa, e o custo aparece na bateria do tablet da
  clínica, longe de quem decidiu.
- **Uma biblioteca de componentes instalada por dependência**, em vez de código copiado · rejeitada:
  copiar é o que permite apagar comentário, ajustar `prefers-reduced-motion` e manter o componente sob as
  regras do repo.

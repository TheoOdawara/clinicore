# 0002. Web e site em Vite, com prerender no site

- Status: accepted, emendada pela 0004 no ferramental de lint e na versão do TypeScript, e pela 0007 no
  offline, que passa a ser exclusivo do app nativo, e pela 0008 no `apps/site`, que passa a Next.js
  standalone
- Date: 2026-09-20

## Context

A ADR 0001 reconstruiu o `apps/api` em NestJS e deixou o front explicitamente pendente em três lugares:
o `CLAUDE.md` chamava os dois gerenciadores de pacote de estado transitório "até o web migrar para
Next"; a própria 0001 registrou que `docs/specs/autenticacao/003-autenticacao-web.md` "segue descrevendo
o cliente do Better Auth" e que isso se resolve quando o front migrar; e as tasks 8 a 11 dessa spec nunca
viraram issue.

A pergunta era se o `apps/web` deveria migrar para Next, e com qual fronteira — cliente puro, ou com
servidor próprio fazendo BFF e renderização.

Três informações mudaram a resposta durante a decisão:

1. **O sistema não tem superfície pública.** `docs/requirements/` diz que o paciente não opera o
   sistema e interage só por WhatsApp e documentos recebidos; a assinatura do paciente é feita em
   plataforma de terceiros; a cobrança do SaaS está fora do escopo. Todo usuário do `apps/web` é
   autenticado.
2. **Vai existir landing pública com SEO**, em domínio próprio, editada em código pelo dono do produto.
3. **A landing é pesada de mídia e de animação** — imagem, vídeo, animação disparada por rolagem — e o
   dono do produto declarou que ela pode vir a virar uma experiência contínua só, em vez de páginas
   independentes.

Os quatro ganhos reais do Next — SEO, `next/image`, Data Cache e ISR — caem todos em superfície pública.
Com a landing virando app próprio, nenhum deles alcança o sistema.

## Decision

O **`apps/web` permanece em Vite 8, React 19.3 e TanStack Router 1.170**, com Bun, Biome, TypeScript 7.0
e Jest como estão. Ele segue sendo cliente puro: o browser fala direto com a API, em origem cruzada, com
`credentials: "include"`.

Nasce o **`apps/site`, na mesma stack do `apps/web`**, para a landing pública: Vite 8, React 19.3,
TanStack Router 1.170, Bun, Biome e TypeScript 7.0. O que ele acrescenta são dois plugins de build:

| Papel | Pacote | Versão | Peer |
| --- | --- | --- | --- |
| HTML pronto no build, para indexação | `vite-prerender-plugin` | 0.5.13 | `vite 5.x \|\| 6.x \|\| 7.x \|\| 8.x` |
| Compressão e conversão de imagem | `vite-imagetools` | 12.0.1 | `vite >=8.0.0` |

| | `apps/site` | `apps/web` | `apps/api` |
| --- | --- | --- | --- |
| Domínio | `clinicore.com.br` | `app.clinicore.com.br` | `api.clinicore.com.br` |
| Público | aberto e indexável | só usuário autenticado | — |
| Entrega | estático, HTML gerado no build | estático, PWA instalável | processo Node |

Decisões de detalhe dentro desta:

- **O `apps/site` usa a mesma stack do `apps/web` porque a landing pode ser absorvida por ele.** O dono
  do produto pediu explicitamente para não ficar com uma migração pendente no futuro. Duas stacks
  diferentes transformariam essa fusão numa reescrita; a mesma stack a transforma em mover pasta.
- **O SEO vem de prerender no build, não de servidor.** O `vite-prerender-plugin` chama uma função
  `prerender()` exportada pelo app, que renderiza o router com `createMemoryHistory` e devolve
  `{ html, links, head }`; o plugin grava o HTML de cada rota e segue os `links` para achar as outras.
  Saída é o mesmo `dist/` estático, sem runtime e sem servidor.
- **axios entra no `apps/web`.** `shared/http/` passa a ser uma instância com `baseURL`,
  `withCredentials: true` e o interceptor de `401` que faz o refresh — que o cliente do Better Auth
  escondia e agora é código nosso. O Zod continua validando toda resposta.
- **O Better Auth sai do web**, porque saiu da API na 0001. Toda chamada passa a bater nas rotas
  `/auth/*` descritas em `docs/specs/autenticacao/003-autenticacao-api.md`.
- **A API passa a ter duas origens.** `APP_ORIGIN` monta os redirecionamentos de autenticação e aponta
  só para o sistema; `ALLOWED_ORIGINS` é a lista que o CORS e a guarda de `Origin` conferem por
  pertencimento, e inclui a landing, que pode vir a chamar a API.
- **Os cookies de sessão passam a `SameSite=Lax` em todo ambiente.** `app.clinicore.com.br` e
  `api.clinicore.com.br` têm o mesmo domínio registrável, e `SameSite` é calculado por domínio
  registrável, não por origem: são o mesmo site. A regra 2 da spec da API afirmava `None` em produção
  "porque o web e a API ficam em subdomínios distintos e o cookie viaja entre sites", e a justificativa
  estava errada. A mesma afirmação no corpo da issue #4 é corrigida junto.
- **O `apps/site` não tem CMS.** O dono edita a landing no código, então ela é site estático gerado do
  repo, sem painel e sem banco.
- **O repo passa a ter três apps independentes**, cada um com o próprio manifesto e o próprio lockfile.
  Os dois gerenciadores de pacote — npm na API, Bun no web e no site — deixam de ser estado transitório e
  passam a ser estado decidido.

Esta decisão **resolve a pendência que a 0001 deixou** sobre `003-autenticacao-web.md` e emenda a seção
Architecture do `CLAUDE.md`, que é reescrita junto.

## Consequences

Fica mais fácil:

- O SEO deixa de pressionar o sistema. A landing é estática, indexável e tem cadência de deploy própria.
- **Uma stack só no front.** Mesmo lint, mesmo formatador, mesmo gerenciador, mesma versão de TypeScript,
  e componente que nasce num app roda no outro sem tradução. Fundir landing e sistema, se isso for
  decidido, é mover pasta.
- O `apps/web` não muda de arquitetura: as sete telas da spec de autenticação são escritas uma vez só.
- A guarda de `Origin` deixa de ser a única linha contra `POST` de outro site, porque `Lax` já não
  carrega o cookie nesse caso. Ela continua, como segunda linha.
- Nenhum runtime novo enxerga prontuário. O dado de saúde continua tocando browser e API, e mais nada.

Fica mais difícil, e é aceito:

- **O repo ganha um terceiro app**, com o terceiro job de CI e o terceiro gate a manter verde.
- **O runtime do React vai em toda página da landing.** São cerca de 45 KB comprimidos que um gerador de
  site estático não cobraria. Numa landing de imagem, vídeo e animação o custo é ruído ao lado da mídia;
  numa página só de texto seria peso desnecessário, e é o preço da stack única.
- **O prerender pode quebrar em silêncio, e por isso vira gate de CI.** Um gerador de site estático
  sempre emite HTML; aqui o HTML só existe se a função `prerender()` continuar rodando, e um componente
  que toca `window` fora de efeito derruba ela deixando o build verde com HTML vazio. O job do site
  confere, depois do build, que uma frase conhecida da landing está dentro do `dist/index.html`.
- **As dimensões da imagem passam a ser responsabilidade do componente.** O `vite-imagetools` comprime,
  redimensiona e converte para AVIF e WebP com `sharp`, mas não escreve `width` e `height` no HTML. O
  `CLS = 0` que o contrato global exige é escrito à mão no componente de imagem da landing.
- **O refresh do access token vira código nosso**, no interceptor do axios: uma tentativa por `401`, com
  as chamadas concorrentes na fila. O Better Auth fazia isso escondido.
- **`WEB_ORIGIN` deixa de existir** e é substituída por duas variáveis. A issue #65, que ainda não
  começou, e o job da API no CI passam a nomear `APP_ORIGIN` e `ALLOWED_ORIGINS`.
- **O TanStack Router é ecossistema menor que o do Next**, e essa é a contrapartida assumida por não
  migrar.

Gatilho que reabre esta decisão: precisar de renderização no servidor, seja **dentro da área logada**,
seja na landing. A saída então é o **TanStack Start 1.168.56**, que declara `@tanstack/react-router`
1.170.38 como dependência — a mesma linha que os dois apps já usam —, e que dá SSR e server functions sem
trocar de router nem reescrever rota.

Superfícies futuras já avaliadas que **não** reabrem esta decisão, por serem API e arquitetura e não
renderização: código de convite para promoção, que é geração e validação na API com formulário no web; e
portal do paciente com prontuário online, que nasce como app próprio, por ter audiência, autenticação,
modelo de ameaça e deploy diferentes do sistema da clínica.

## Alternatives considered

- **Astro 7.3.3 no `apps/site`** · rejeitada pelo dono do produto, e a troca é medida. O Astro entregava
  zero JavaScript por padrão, `astro:assets` com `sharp` como `optionalDependency` do próprio pacote, e
  `Image`, `Picture`, `ResponsiveImage` e `Font` embutidos escrevendo `width` e `height` sozinhos. Em
  troca, cobrava uma segunda stack: `.astro` em vez de `.tsx`, ilha `client:visible` como fronteira de
  interatividade, e **TypeScript 6.0 obrigatório**, porque o gate de tipos é o `astro check`, que exige
  `@astrojs/check` à parte com peer `typescript: "^5.0.0 || ^6.0.0"` — por baixo é Volar, que usa a API
  programática do compilador, a mesma que o 7.0 não tem. Duas fraquezas decidiram: uma timeline de
  rolagem que atravessa várias seções não reparte bem entre ilhas React, que são raízes isoladas sem
  contexto comum; e a fusão com o `apps/web`, declarada como possível, seria reescrita e não mudança de
  pasta. O que se perdeu ao recusar está registrado em Consequences como custo aceito.
- **Migrar o `apps/web` para Next 16 exportado estático** · rejeitada: `output: "export"` desliga
  `next/image`, Route Handler e middleware, e o `@serwist/next` 9.5.12 é plugin de webpack que não
  suporta Turbopack — obriga `next dev --webpack`. Paga-se a migração e desliga-se metade do que veio com
  ela; sobra router diferente e `next/font`. O ferramental não era impeditivo: TypeScript 7.0, Bun, Biome
  e Jest ficariam de pé, porque `experimental.useTypeScriptCli` é `true` por padrão no `next@16.3.5` e
  dispara o `tsc` do projeto, o Biome 2.5 tem domínio `next`, e o `create-next-app` não configura Jest.
- **Next 16 com BFF ou RSC** · rejeitada: os cookies de sessão são `HttpOnly` do host da API e sem
  `Domain`, então servidor nenhum do web os alcança sem proxy. Fazer o proxy quebra o link de verificação
  de e-mail e o callback do Google, que são navegações até a API que gravam cookie lá — seria preciso
  mudar o `redirect_uri` registrado no Google Cloud Console e o host do cookie de `state`. Além disso
  coloca prontuário num segundo runtime, com cache de fetch, cache de rota, log e disco, e autorização
  no framework é a classe do bypass de middleware que o Next já teve. O ganho seria o primeiro paint
  frio, que o precache do PWA neutraliza no uso diário.
- **TanStack Start agora, em qualquer um dos dois apps** · rejeitada: mesmo ganho de servidor com os
  mesmos custos, sem necessidade hoje. Fica nomeada como a saída se o gatilho acima disparar.
- **A landing dentro do `apps/web`** · rejeitada: o `apps/web` é área logada com service worker
  precacheando o shell, e o contrato de mídia da landing é o oposto do contrato de dado clínico. Misturar
  as duas põe os dois bundles no mesmo build e o service worker no mesmo escopo. Apps separados na mesma
  stack dão a fusão futura sem pagar a confusão agora.
- **`next/image` para as fotos do paciente** · rejeitada: o otimizador busca a imagem, re-encoda e grava
  a variante em disco do servidor do web. Seria dado de saúde em cache num segundo servidor, contra a
  regra 9 da spec do web. Comprimir foto de paciente é trabalho do pipeline de upload na API.
- **`vite-react-ssg` 0.9.2 para o prerender** · rejeitada: declara peer `react-router-dom ^6.14.1`, que é
  outra família de router. Traria um segundo roteador só para gerar HTML.
- **Vike 0.4.266** · rejeitada: é framework completo sobre o Vite, com roteamento e convenção de arquivo
  próprios. Substituiria o TanStack Router para resolver só a geração de HTML.
- **`fetch` em vez de axios no web** · rejeitada pelo dono do produto, que pediu axios. O `fetch` cobriria
  JSON, erro e `AbortSignal.timeout()` em poucas linhas, e o interceptor de refresh existiria de
  qualquer forma no `shared/http/`; em compensação o `fetch` não reporta progresso de upload de forma
  portável, que as issues #22 e #39 vão precisar.
- **Usar axios no web por simetria com o `@nestjs/axios` da API** · rejeitada como justificativa: os apps
  não importam código um do outro, então não há interceptor, instância nem tipo compartilhado, e o
  `@nestjs/axios` serve para chamada de saída a terceiro, que é outro trabalho. O axios entra por decisão
  própria do front, não por simetria.

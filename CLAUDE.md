# Clinicore — contrato do repositório

Aditivo ao contrato global (`~/.claude/CLAUDE.md`); em conflito, **este arquivo vence**. Nada aqui
repete o que já é global.

O produto — problema, escopo, go-live da clínica piloto e critérios de aceite — está em
`docs/requirements.md`. Este arquivo diz **como** o sistema é construído, nunca **o que** ele faz.

## Stack

Três apps **independentes** em `apps/`, cada um com o próprio `package.json` e o próprio lockfile. Sem
manifest na raiz, sem workspaces, sem código ou tipo compartilhado por import. A fronteira entre eles é
o contrato HTTP.

**A API usa npm e os dois apps de front usam Bun.** É estado decidido, não transitório.

Cada app tem o seu domínio: `clinicore.com.br` é a landing pública, `app.clinicore.com.br` é o sistema e
`api.clinicore.com.br` é a API. Os três compartilham o mesmo domínio registrável, e é isso que mantém o
cookie de sessão em `SameSite=Lax`.

| | `apps/api` | `apps/web` | `apps/site` |
|---|---|---|---|
| Papel | o contrato HTTP | sistema da clínica, PWA instalável, só usuário autenticado | landing pública e indexável |
| Runtime e pacotes | Node 26 · npm | Bun 1.4 · Node 26 só para rodar o Jest | Bun 1.4 |
| Tipos | TypeScript 6.0 | TypeScript 7.0 | TypeScript 7.0 |
| Lint e formato | ESLint 10 · `typescript-eslint` 8, com regras type-aware · Prettier 3 | Biome 2.5 | Biome 2.5 |
| Framework | NestJS 11 sobre Express, validação e DTO em `class-validator` e `class-transformer`, OpenAPI por `@nestjs/swagger`, health check por `@nestjs/terminus` | React 19.3 · Vite 8.3 · TanStack Router 1.170 (file-based) | React 19.3 · Vite 8.3 · TanStack Router 1.170, com `vite-prerender-plugin` 0.5 e `vite-imagetools` 12 no build |
| Dados | PostgreSQL 18 · TypeORM 1.1 com `@nestjs/typeorm` e `pg` | TanStack Query 5.102 · axios 1 · Zod 4.6 | — |
| Formulário | — | TanStack Form 1.33 com schema Zod | — |
| Estilo | — | Tailwind 4.3 · shadcn/ui sobre Radix e CVA · `tw-animate-css` | Tailwind 4.3 · shadcn/ui, com cópia própria |
| Auth | `@nestjs/passport`, `@nestjs/jwt` e `@node-rs/argon2`; access token curto e refresh na tabela `session` | axios contra as rotas `/auth/*` da API, com o refresh no interceptor | — |
| Fila e agendamento | `@nestjs/bullmq` · `@nestjs/schedule` | — | — |
| Redis 8 | fila, contagem do limite por IP e denylist de revogação de sessão | — | — |
| HTTP de saída | `@nestjs/axios` sobre axios 1 | — | — |
| Configuração | `@nestjs/config`, validada por `class-validator` no boot | — | — |
| E-mail | Nodemailer pelo SMTP do Gmail (`smtp.gmail.com:587`), em todos os ambientes | — | — |
| Log | Pino 10 atrás de um `LoggerService` do Nest, JSON em stdout, com `redact` | — | — |
| Testes | Jest 30 · `ts-jest` · supertest · `@nestjs/testing` | Jest 30 · `@swc/jest` · jsdom · Testing Library | — |

As decisões que trouxeram esta stack, o que foi descartado e por quê:
`docs/decisions/0001-api-em-nestjs-typeorm-e-redis.md`,
`docs/decisions/0002-web-e-site-em-vite.md` e
`docs/decisions/0003-design-system-com-shadcn-ui-e-react-bits.md`.

## Comandos

Cada comando roda de dentro do diretório do seu app.

| Gate | `apps/api` | `apps/web` | `apps/site` |
|---|---|---|---|
| Análise estática e formato | `npm run lint` · `npm run format:check` | `bun run check` | `bun run check` |
| Tipos | `npm run typecheck` | `bun run typecheck` | `bun run typecheck` |
| Build | `npm run build` | `bun run build` | `bun run build` |
| Testes | `npm run test` · `npm run test:e2e` | `bun run test` | — enquanto não houver lógica a testar |

Instalação: `npm ci` na API, `bun install --frozen-lockfile` no web e no site.

O gate de tipos do web e do site exige o `src/routeTree.gen.ts`, gerado pelo plugin do TanStack Router.
Ele é commitado, então só um `src/routes/` alterado sem `vite build` ou `vite dev` desde a alteração
deixa o `tsc` olhando para uma árvore velha.

**O build do site tem um passo a mais que o `bun run build`:** o job confere que o HTML gerado tem
conteúdo. É o que denuncia um prerender que parou de rodar.

## Arquitetura

```
apps/
├── api/                     NestJS · :3333
│   ├── compose.yaml         Postgres e Redis de desenvolvimento
│   ├── test/                e2e: <name>.e2e-spec.ts e jest-e2e.json
│   └── src/
│       ├── main.ts          entrada HTTP, só boot
│       ├── worker.ts        entrada do worker da fila, só boot
│       ├── app.module.ts    só fiação
│       ├── common/          exceptions, filters, guards, pipes, decorators, types
│       ├── core/            config, db, logger, mail, queue, redis
│       │   ├── core.module.ts       agrega; só o AppModule alcança
│       │   └── db/
│       │       ├── migrations/
│       │       └── data-source.ts   usado pela CLI do TypeORM
│       └── features/<feature>/
│           ├── <feature>.module.ts      único arquivo solto na raiz
│           ├── controller/
│           ├── service/
│           ├── repository/
│           ├── dto/
│           ├── entities/
│           ├── enums/ constants/ utils/ job/ strategy/   só quando houver conteúdo
│           └── __tests__/
├── web/                     Vite · :3000 · app.clinicore.com.br
│   └── src/
│       ├── main.tsx
│       ├── routes/          roteamento, loader, composição
│       ├── features/<feature>/
│       │   ├── api/         chamada axios, schema Zod da resposta, queryOptions, texto de erro
│       │   ├── components/
│       │   ├── hooks/
│       │   └── __tests__/
│       ├── styles/          globals.css é manifesto; regra por concern em arquivo próprio
│       └── shared/          UI base do shadcn, http, env — sem regra de negócio
└── site/                    Vite · :4321 · clinicore.com.br
    └── src/
        ├── main.tsx
        ├── prerender.tsx    exporta prerender(); só o build a chama
        ├── routes/          uma página por arquivo, prerenderizada
        ├── sections/        blocos da landing: hero, preços, dúvidas
        ├── components/      UI base do shadcn e o que for copiado
        ├── assets/          imagem e vídeo da landing
        └── styles/
compose.yaml                 stack inteira: web, api, worker, Postgres e Redis
docs/
```

**O browser fala direto com a API, em origem cruzada.** Em dev são as portas :3000 e :3333; em homolog e
produção, `app.clinicore.com.br` e `api.clinicore.com.br`. Por isso o `enableCors()` da API libera as
origens de `ALLOWED_ORIGINS` com `credentials: true`, e o web chama com `withCredentials: true`.

**`apps/site` é estático e não tem servidor.** Ele não participa de sessão nem lê cookie; se um dia
chamar a API — formulário de contato, pedido de demonstração, código de convite —, é por rota pública e
a origem dele entra em `ALLOWED_ORIGINS`. Redirecionamento de autenticação nunca aponta para ele.

### `api` — camadas

**A dependência aponta para baixo: `Controller → Service → Repository → TypeORM`.** Quem a faz valer é
o container de DI, não convenção: o que não está nos `providers` do módulo não é alcançável.

- **Módulo** é o `<feature>.module.ts`, o único arquivo solto na raiz da feature. Declara
  controller, service e repository, registra as entities com `TypeOrmModule.forFeature()`, e
  exporta só o service quando outra feature precisa dele. `core.module.ts` é alcançado apenas pelo
  `AppModule`.
- **Subpasta existe quando tem conteúdo.** `enums/`, `constants/`, `utils/`, `job/` e `strategy/` só
  nascem com o primeiro arquivo; pasta vazia não é reservada.
- **Strategy do Passport é da feature, guard é de `common/`.** A `strategy/` da feature declara como um
  credencial vira usuário; o guard que a consome vale para toda feature, é registrado como `APP_GUARD`
  e mora em `common/guards/`, junto dos decorators que o acompanham.
- **Controller** é a classe `@Controller()` em `controller/`: rota, DTO de entrada e saída, chamada
  ao service. Não tem regra.
- **Service** é a regra de negócio, em `service/`. **Não importa `typeorm`, não usa
  `@InjectRepository` e não abre transação** — nem para uma consulta trivial. Quebrar isso é como o
  service vira arquivo de mil linhas.
- **Um service por operação ou grupo coeso de operações**, nunca um por feature. O nome diz qual:
  `protocol-archive.service.ts`, não um `protocol.service.ts` com vinte métodos públicos. Se os
  testes de um service precisam ser divididos por operação, o service já devia estar dividido.
- **Repository** é a classe `@Injectable()` em `repository/`, a única que injeta `EntityManager` ou
  `Repository<Entity>`, junto com `core/db/`. **Transação só aqui**: operação que grava em mais de
  uma tabela é um método de repository que abre o `dataSource.transaction()` dentro dele.
- **Entity** é a classe TypeORM em `entities/` da própria feature, uma por tabela.
- **A feature se divide quando acumula um segundo substantivo.** Anexo, nota e associação de um
  protocolo são features próprias, não subpastas dele. O gatilho é responsabilidade misturada; uma
  `dto/` com trinta arquivos é o alarme, não a regra.
- **DTO** é a classe com decorators de `class-validator` em `dto/`; o OpenAPI sai dela pelo plugin de
  CLI do `@nestjs/swagger`, sem `interface` paralela.
- **Não há dono de tabela.** O repository de uma feature lê e escreve a tabela que a operação dela
  precisa.
- **Só a API do TypeORM** (`find`, `findOne`, `relations`, `QueryBuilder`, `count` e afins). **Raw SQL é
  proibido, sem exceção**: `DataSource.query`, `EntityManager.query`, `QueryRunner.query` e qualquer
  trecho de SQL cru dentro de `QueryBuilder`. Agregação com join é composta no service a partir de
  chamadas separadas ao repository.
- **Relação vem por `relations`, e lote vem por `In()`.**
- **Erro:** o service lança `BusinessError` com um tipo (`NotFound`, `Conflict`, `Forbidden`,
  `Invalid`, `Unauthorized`) e um código; o repository traduz o erro conhecido do TypeORM — `QueryFailedError` com o
  código do Postgres, `EntityNotFoundError` — para esses tipos. Só o `ExceptionFilter` global em
  `common/filters/` conhece HTTP: converte o tipo em status e responde `{ code, message, fields }`.
  Erro desconhecido vira 500 sem detalhe.
- **A `message` da API é inglês e é texto de desenvolvedor**, para log e depuração. O que o usuário lê
  é escrito no web, a partir do `code`.
- **Log:** `core/logger/` cria a instância do Pino e o `LoggerService` registrado por
  `app.useLogger()`, mais o interceptor que registra a requisição. **Nenhum wrapper de terceiro.** O
  nível vem de `LOG_LEVEL`, obrigatória como toda variável. **O `/health` não é logado**, porque quem o
  chama é o orquestrador, a cada poucos segundos. **Nada de segredo sai no log:** header de
  autorização, cookie, senha e connection string passam pelo `redact` do Pino. O Pino escreve em
  `stdout`; o erro de ambiente do boot continua indo cru para `stderr`, antes de existir logger.
- **Ambiente:** `core/config/` registra o `@nestjs/config` com uma classe validada por
  `class-validator` no boot. Toda variável é obrigatória, **sem default no ponto de leitura**. A
  leitura acontece por um acessor tipado, nunca por `ConfigService.get` direto.
- **Fila:** `core/queue/` registra o `@nestjs/bullmq` contra o Redis; o service enfileira pela fila
  injetada com `@InjectQueue`; o job é um `@Processor` em `features/<feature>/job/` e chama o service,
  como o controller faz. **O worker é um processo separado** (`worker.ts`), com o seu próprio
  container.
- **Testes** ficam em `__tests__/` da feature. O teste padrão é um por comportamento, subindo o módulo
  com `Test.createTestingModule` e batendo na rota com `supertest` contra o Postgres real. O e2e que
  sobe o `AppModule` inteiro fica em `test/`. Teste unitário existe só para cálculo puro (parcelamento,
  repasse). Não se faz mock de repository nem de `DataSource`.

### `web` — por feature

**A dependência aponta para cima: `shared → features → routes`.**

- **Rota é fina:** declara o caminho, carrega dado no `loader` com
  `queryClient.ensureQueryData(<feature>QueryOptions)`, define `pendingComponent` e
  `errorComponent`, e compõe features. `routes/(app)/route.tsx` exige sessão.
- **Uma feature não importa outra.** Quem junta duas features é a rota.
- **`shared/` não tem regra de negócio.** Componente de domínio mora na feature.
- **Sem barrel file** (`index.ts` que só reexporta).
- Arquivo usado por uma única rota pode ficar colocado ao lado dela, em pasta com prefixo `-`
  (fora do roteamento).
- **`queryOptions` fica em `features/<feature>/api/`**, e o componente lê com `useSuspenseQuery`.
- **Toda resposta da API passa por `.parse()` de um schema Zod** em `features/<feature>/api/`, e o
  tipo sai de `z.infer`. O schema replica o DTO da API à mão.
- **`shared/http/` é a única instância do axios**, com `baseURL`, `withCredentials: true` e o
  interceptor que renova a sessão: um `401` dispara uma única chamada a `POST /auth/refresh`, as
  requisições concorrentes esperam essa chamada em vez de dispararem a sua, e o fracasso leva para
  `/login`. Nenhuma feature cria instância própria nem chama `axios` direto.
- **Formulário é TanStack Form** com o schema Zod nos validadores de blur e submit.
- **A UI base é shadcn/ui**, copiada para `shared/`, e o que o shadcn cobre não é reescrito à mão. O
  React Bits entra pela tabela de categorias da ADR 0003 — no `apps/web` não há fundo animado, efeito
  de cursor nem texto animado sobre dado clínico. Todo componente copiado respeita
  `prefers-reduced-motion`, nunca atrasa informação na tela, e perde os comentários na mesma edição.
- **O texto de erro que o usuário lê é pt-BR e mora no web**, traduzido a partir do `code` que a API
  devolve, dentro de `features/<feature>/api/`. `code` sem tradução cai numa mensagem genérica.
- **Testes:** lógica pura (schema, formatação, hook) e componente que decide algo (formulário, estado
  vazio ou de erro), com `features/*/api` substituído por `jest.fn`. Componente que só exibe não
  tem teste. Ficam em `__tests__/` ao lado do código testado, nos dois apps.
- **URL é inglês**, porque o nome da pasta de rota é o caminho: `/patients`, nunca `/pacientes`.

### `site` — estático

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

## Branches

- **`main` é produção e `develop` é homolog, as duas protegidas por convenção** — nunca commitar
  direto nelas. O GitHub não aplica a proteção: o repo é privado numa conta sem GitHub Pro, e a API de
  proteção e de rulesets responde 403.
- Cada entrega nasce em `feature/<número>-<assunto>` a partir de `develop` e volta para ela por pull
  request. Correção urgente é `hotfix/<número>-<assunto>`, a partir da `main`.
- Release é pull request de `develop` para `main`.
- **A mensagem de commit é só o título.** O porquê e o detalhe vão na descrição do pull request.
- **CI roda em pull request para `develop` e para `main`**, por `.github/workflows/ci.yml`, com um job por app.

## Idioma

**Só a documentação é pt-BR.** Código, identificadores, pastas, arquivos e mensagens de commit são
inglês. O chat segue em pt-BR. **A URL é a única coisa que depende do app**, e cada seção da
arquitetura diz a sua.

## Pegadinhas da stack

As do web foram verificadas em 2026-09-15; as da API, em 2026-09-19, contra as versões desta stack e
antes de existir código em NestJS.

### `apps/api`

- **O NestJS 12 é ESM-only** — o `@nestjs/common@12` publica `"type": "module"` e o 11 não. Um Jest em
  CommonJS não carrega ESM do `node_modules` e morre com `Must use import to load ES Module`. A saída
  seria `--experimental-vm-modules`, e o `vm.SourceTextModule` ainda é Stability 1 na documentação do
  Node 26, sem release alvo para sair. **O repo fica no Nest 11 por causa disso.**
- **O TypeScript 7.0 está fora da API por duas vias.** O `@nestjs/cli@11.0.24` carrega
  `typescript 5.9.3` como dependência direta, e o `ts-jest@29` declara peer `typescript >=4.3 <7`. O
  `apps/web` segue no 7.0 porque usa `@swc/jest` e não passa por nenhum dos dois.
- **`ConfigService.get` devolve `any` sem `{ infer: true }`.** A sobrecarga que casa com
  `config.get('CHAVE')` tem `T = any`, e tipar o serviço como `ConfigService<Env, true>` não ajuda —
  `K` restringe só o nome da chave. `const port: number = config.get('PORT')` compila com `PORT`
  string. Por isso a leitura passa por um acessor tipado sobre `getOrThrow`, nunca por `get` direto.
- **Os pacotes-satélite do Nest saltaram a numeração para acompanhar o core.** O `@nestjs/config` foi
  de `4.0.4` para `12.0.0` sem nada entre os dois, e o mesmo vale para `@nestjs/schedule` e
  `@nestjs/event-emitter`. O peer deles é `@nestjs/common: ^11.0.0 || ^12.0.0`, então a versão 12
  desses pacotes roda sobre o Nest 11. **O `@nestjs/swagger` é a exceção e não generaliza:** o
  `@nestjs/swagger@12` exige `@nestjs/common: ^12.0.0` e o npm recusa a instalação sobre o Nest 11 —
  o pin é `@nestjs/swagger@11.4.7`. O `@nestjs/terminus@12` aceita as duas linhas e entra normal.
- **O `latest` do TypeORM é o 1.1.1, e o 0.3.x virou o dist-tag `legacy`.** Tutorial e resposta de
  fórum anteriores a isso descrevem a API do 0.3.
- **A DI do Nest depende de `reflect-metadata` e de `emitDecoratorMetadata`.** Faltando qualquer um
  dos dois, a compilação passa e a injeção falha em runtime.
- **O `class-validator` não tem decorador de CIDR.** Tem `@IsIP`, `@IsPort`, `@IsUrl`, `@IsFQDN` e
  `@IsEmail`, mas nada de faixa — e `TRUSTED_PROXIES` precisa. O `validator` 13, que o próprio
  `class-validator` traz como dependência, tem `isIPRange`: o decorador sai de um `registerDecorator`
  de poucas linhas, nunca de regex à mão.
- **O `StandardSchemaValidationPipe` existe no `@nestjs/common@11` e não dá para usar.** Ele lê
  `metadata.schema`, e no 11 o `ArgumentMetadata` não tem esse campo nem o `@Body()` tem sobrecarga que
  o alimente. É encanamento adiantado para o 12. Conferir que o arquivo existe não basta.

### `apps/web`

- **O Vite não checa tipo.** Um arquivo com erro de tipo roda e sai com código 0; só o `tsc` pega.
- **O TypeScript 7.0 não tem a API programática do compilador** (prevista para a 7.1). Por isso o
  `ts-jest` e os geradores de tipo a partir de OpenAPI quebram
  (`Cannot read properties of undefined (reading 'createKeywordTypeNode')`). O web usa `@swc/jest`.
  Ferramenta que exija essa API roda com `typescript` apontado para `@typescript/typescript6`, e o
  `tsc` 7 continua em `@typescript/native`.
- **`import.meta.env` do Vite não existe no Jest.** No modo CommonJS, o arquivo que o lê derruba a
  suíte com `Must use import to load ES Module`; no modo ESM, carrega e o valor chega `undefined`.
- **O binário do Jest é `#!/usr/bin/env node`**: mesmo com Bun, o web precisa de Node instalado.
- **O Biome com `vcs.useIgnoreFile` procura o `.gitignore` na pasta onde está o `biome.json`**, não na
  raiz do repositório, e aborta a execução inteira se não achar. Cada app que usa Biome precisa do seu.
- **O `shadcn add` e o `jsrepo` geram arquivo comentado**, e o repo só admite o marcador `ponytail:`. A
  limpeza é parte de adicionar o componente, não uma passada depois.
- **Parte dos componentes do React Bits ignora `prefers-reduced-motion`.** Conferir e corrigir no
  arquivo copiado; não dá para presumir que a fonte respeitou.

### `apps/site`

- **Build verde não significa página indexável.** O `vite-prerender-plugin` chama a `prerender()` do
  app; se ela lançar, o que sobra é o `index.html` do Vite, que é uma `<div>` vazia. O `tsc` não pega,
  o Biome não pega e o build sai com código 0. Por isso o gate do site confere o conteúdo do HTML
  gerado, e não só que o build passou.
- **`window` fora de efeito é o jeito mais comum de derrubar o prerender**, porque a `prerender()` roda
  em Node. Vale para código copiado do React Bits, que costuma ler `window` na montagem.
- **O `vite-imagetools` não escreve `width` nem `height`.** Ele entrega o arquivo otimizado e mais nada;
  a dimensão no HTML é trabalho do componente.

### Infraestrutura

- **O Gmail reescreve o `From` com a conta autenticada, em silêncio.** Mandar
  `from: "nao-responda@clinicore.app"` autenticado como outra conta não falha: chega com o endereço
  da conta. Alias do "Enviar e-mail como" não vale no `smtp.gmail.com`. Sair disso exige o
  `smtp-relay.gmail.com`, que é Google Workspace, não conta comum.
- **A autenticação é por app password, não pela senha da conta.** Criar uma exige verificação em duas
  etapas ligada, e o acesso a "apps menos seguros" acabou em 2025-05-01.
- **O limite de envio é diário e da conta inteira** — 2.000 mensagens por dia no Workspace, menos numa
  conta comum — e dev e homolog dividem o mesmo teto se apontarem para a mesma conta.

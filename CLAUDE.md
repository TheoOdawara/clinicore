# Clinicore — contrato do repositório

Aditivo ao contrato global (`~/.claude/CLAUDE.md`); em conflito, **este arquivo vence**. Nada aqui
repete o que já é global.

O produto — problema, escopo, go-live da clínica piloto e critérios de aceite — está em
`docs/requirements.md`. Este arquivo diz **como** o sistema é construído, nunca **o que** ele faz.

## Stack

Dois apps **independentes** em `apps/`, cada um com o próprio `package.json` e `bun.lock`. Sem manifest
na raiz, sem workspaces, sem código ou tipo compartilhado por import. A fronteira entre eles é o
contrato HTTP.

| | `apps/api` | `apps/web` |
|---|---|---|
| Runtime e pacotes | Bun 1.4 | Bun 1.4 · Node 26 só para rodar o Jest |
| Tipos | TypeScript 7.0 | TypeScript 7.0 |
| Lint e formato | Biome 2.5 | Biome 2.5 |
| Framework | Elysia 1.4, validação e DTO em TypeBox, OpenAPI por `@elysiajs/openapi` | React 19.3 · Vite 8.3 · TanStack Router 1.170 (file-based) |
| Dados | PostgreSQL 18 · Prisma 7.10 com `@prisma/adapter-pg` | TanStack Query 5.102 · `fetch` nativo · Zod 4.6 |
| Formulário | — | TanStack Form 1.33 com schema Zod |
| Estilo | — | Tailwind 4.3 |
| Auth | Better Auth 1.7 | cliente do Better Auth |
| Fila e agendamento | pg-boss 12, no próprio Postgres | — |
| E-mail | Nodemailer pelo SMTP da Resend, em todos os ambientes | — |
| Testes | `bun test` | Jest 30 · `@swc/jest` · jsdom · Testing Library |

**Fora da stack, por decisão:** Redis, RabbitMQ e qualquer camada de cache; Axios; Kysely ou outro
query builder; tipos gerados a partir do OpenAPI; Eden Treaty (exige importar o tipo do servidor, e
os apps não compartilham código).

## Comandos

**`apps/web` ainda não foi criado.** A ferramenta de cada gate já está decidida; a invocação exata
entra aqui quando o app nascer, depois de rodar. Cada comando roda de dentro do diretório do seu app.

| Gate | `apps/api` | `apps/web` |
|---|---|---|
| Análise estática e formato | `bun run check` | Biome |
| Tipos | `bun run typecheck` | `tsc` sem emitir |
| Build | `bun run build` | Vite |
| Testes | `bun run test` | Jest |

## Arquitetura

```
apps/
├── api/                     Elysia · :3333
│   ├── compose.yaml
│   ├── prisma/
│   │   ├── schema.prisma    todas as entities
│   │   └── migrations/
│   └── src/
│       ├── server.ts        entrada HTTP
│       ├── worker.ts        entrada da fila
│       ├── common/          dto, errors, types compartilhados
│       ├── core/            config, db, mail, queue
│       └── features/<feature>/
│           ├── controller/<feature>.controller.ts
│           ├── service/<feature>.service.ts
│           ├── repository/<feature>.repository.ts
│           ├── job/<name>.job.ts
│           ├── dto/<name>.dto.ts
│           └── __tests__/
└── web/                     Vite · :3000
    └── src/
        ├── main.tsx
        ├── routes/          roteamento, loader, composição
        ├── features/<feature>/
        │   ├── api/         fetch, schema Zod da resposta, queryOptions, texto de erro
        │   ├── components/
        │   ├── hooks/
        │   └── __tests__/
        └── shared/          UI base, http, env — sem regra de negócio
compose.yaml                 stack inteira: web, api, worker e Postgres
docs/
```

**O browser fala direto com a API, em origem cruzada.** Em dev são as portas :3000 e :3333; em
homolog e produção, subdomínios. Por isso o CORS da API libera a origem exata do web com
credenciais, o Better Auth declara a mesma origem em `trustedOrigins`, e o web chama com
`credentials: "include"`.

### `api` — camadas

**A dependência aponta para baixo: `Controller → Service → Repository → Prisma`.**

- **Controller** é a instância Elysia da feature: rota, DTO de entrada e saída, chamada ao service.
  Não tem regra.
- **Service** é a regra de negócio. Não importa `elysia`, o client do Prisma nem nada de
  `generated/prisma`.
- **Repository** é a única camada que importa o client do Prisma, junto com `core/db/`.
- **DTO** é o schema TypeBox em `dto/`; o tipo sai do próprio schema, sem `interface` paralela.
- **Entity** é o model em `prisma/schema.prisma`, um arquivo só. Não existe classe de domínio nem
  mapper.
- **Não há dono de tabela.** O repository de uma feature lê e escreve a tabela que a operação dela
  precisa.
- **Transação só no repository.** Operação que grava em mais de uma tabela é um método de repository
  que abre o `$transaction` dentro dele.
- **Só a API do Prisma** (`findMany`, `include`, `groupBy`, `aggregate` e afins). **Raw SQL é proibido,
  sem exceção**: `$queryRaw`, `$executeRaw`, as variantes `Unsafe`, TypedSQL e extensão que execute
  por esse caminho. Agregação com join é composta no service a partir de consultas separadas.
- **Relação vem por `include`, e lote vem por `in`.**
- **Erro:** o service lança `BusinessError` com um tipo (`NotFound`, `Conflict`, `Forbidden`,
  `Invalid`) e um código; o repository traduz o erro conhecido do Prisma para esses tipos. Só o
  `onError` global em `core/` conhece HTTP: converte o tipo em status e responde
  `{ code, message, fields }`. Erro desconhecido vira 500 sem detalhe.
- **A `message` da API é inglês e é texto de desenvolvedor**, para log e depuração. O que o usuário lê
  é escrito no web, a partir do `code`.
- **Fila:** `core/queue/` conecta o pg-boss ao Postgres; o service enfileira por ele; o job fica em
  `features/<feature>/job/` e chama o service, como o controller faz. **O worker é um processo
  separado** (`worker.ts`), com o seu próprio container.
- **Testes** ficam em `__tests__/` da feature. O teste padrão é um por comportamento, batendo na rota
  com `app.handle(new Request("http://localhost/..."))` contra o Postgres real. Teste unitário existe
  só para cálculo puro (parcelamento, repasse). Não se faz mock de Prisma nem de repository.

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
- **Formulário é TanStack Form** com o schema Zod nos validadores de blur e submit.
- **O texto de erro que o usuário lê é pt-BR e mora no web**, traduzido a partir do `code` que a API
  devolve, dentro de `features/<feature>/api/`. `code` sem tradução cai numa mensagem genérica.
- **Testes:** lógica pura (schema, formatação, hook) e componente que decide algo (formulário, estado
  vazio ou de erro), com `features/*/api` substituído por `jest.fn`. Componente que só exibe não
  tem teste. Ficam em `__tests__/` ao lado do código testado, nos dois apps.
- **URL é inglês**, porque o nome da pasta de rota é o caminho: `/patients`, nunca `/pacientes`.

## Branches

- **`main` é produção e `develop` é homolog, as duas protegidas por convenção** — nunca commitar
  direto nelas. O GitHub não aplica a proteção: o repo é privado numa conta sem GitHub Pro, e a API de
  proteção e de rulesets responde 403.
- Cada entrega nasce em `feature/<número>-<assunto>` a partir de `develop` e volta para ela por pull
  request. Correção urgente é `hotfix/<número>-<assunto>`, a partir da `main`.
- Release é pull request de `develop` para `main`.
- **A mensagem de commit é só o título.** O porquê e o detalhe vão na descrição do pull request.
- **CI roda em pull request para `develop` e para `main`.** Nenhum workflow existe ainda.

## Idioma

**Só a documentação é pt-BR.** Código, identificadores, pastas, arquivos, URLs e mensagens de commit
são inglês. O chat segue em pt-BR.

## Pegadinhas da stack

Verificadas em 2026-09-15 contra as versões desta stack, antes de existir código.

- **Bun e Vite não checam tipo.** Um arquivo com erro de tipo roda e sai com código 0; só o `tsc` pega.
- **O TypeScript 7.0 não tem a API programática do compilador** (prevista para a 7.1). Por isso o
  `ts-jest` e os geradores de tipo a partir de OpenAPI quebram
  (`Cannot read properties of undefined (reading 'createKeywordTypeNode')`). O web usa `@swc/jest`.
  Ferramenta que exija essa API roda com `typescript` apontado para `@typescript/typescript6`, e o
  `tsc` 7 continua em `@typescript/native`.
- **`import.meta.env` do Vite não existe no Jest.** No modo CommonJS, o arquivo que o lê derruba a
  suíte com `Must use import to load ES Module`; no modo ESM, carrega e o valor chega `undefined`.
- **O binário do Jest é `#!/usr/bin/env node`**: mesmo com Bun, o web precisa de Node instalado.
- **Extensão de query do Prisma não vê relação.** Um hook `$allModels.$allOperations` intercepta cada
  operação, inclusive dentro de `$transaction`, mas não a relação carregada por `include` nem o filho
  criado por escrita aninhada.
- **`prisma-extension-kysely` executa por `$queryRawUnsafe` e `$executeRawUnsafe`**, então cai na
  proibição de raw SQL e escapa de qualquer extensão de query.
- **O `by` do `groupBy` aceita só campo escalar do próprio model**; campo de relação é erro de tipo.
  Duas consultas compostas no service ficaram em 2 queries com 10 e com 500 registros; o N+1 foi de 11
  para 501.
- **O dist-tag `latest` do Prisma no npm aponta para `8.0.0-rc`.** O estável é o 7.10, instalado com
  versão explícita. O Prisma 8 lê o schema de um arquivo só.
- **`bun test` com `CLAUDECODE=1` no ambiente esconde os testes que passam**, e a saída capturada por
  um agente não é a do terminal. Com `--parallel`, cada arquivo ganha um global novo, o que muda o
  isolamento de testes que compartilham o banco.
- **A Resend sem domínio verificado só entrega no e-mail da própria conta**, com cota grátis de 100 por
  dia dividida entre dev e homolog.
- **`Value.Convert` do TypeBox arredonda em silêncio.** `PORT=3333.5` contra um `Type.Integer` vira
  `3333` e passa na validação. Onde a coerção importa, a conversão é explícita, não pela biblioteca.
- **O Biome com `vcs.useIgnoreFile` procura o `.gitignore` na pasta onde está o `biome.json`**, não na
  raiz do repositório, e aborta a execução inteira se não achar. Cada app tem o seu.

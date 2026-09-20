# 002 — Criar os dois apps com gates verdes e CI

> **Status:** publicada
> **Perfil:** API
> **Módulo:** `apps/api`, `apps/web`, `.github/workflows`
> **Epic:** #1 — Plataforma
> **Issue:** #2, reescrita para a stack de #72

## Acceptance Criteria

### Contrato

| Método | Rota | Auth / Role | Idempotente |
| --- | --- | --- | --- |
| `GET` | `/health` | público, sem sessão | Sim |

### Request

Sem parâmetros de rota, de query ou de corpo.

### Response

**`200 OK`**

```json
{ "status": "ok" }
```

| Status | Quando |
| --- | --- |
| `200` | O processo da API está de pé e respondendo |

Não há resposta de erro: a rota não depende de banco, de fila nem de sessão. Se o processo estiver fora,
não há resposta — é exatamente essa a informação que o endpoint carrega.

### Perfis e privilégios

| Papel | Permissão | Observação |
| --- | --- | --- |
| Qualquer origem | — | `/health` é público por desenho: quem consome é o orquestrador de container, antes de existir sessão |

Nenhum outro papel existe nesta entrega. Papéis e permissões nascem no epic #5.

---

## Regras de Negócio

### 1. Os dois apps são independentes

- `apps/api` e `apps/web` têm cada um o próprio `package.json` e o próprio lockfile —
  `package-lock.json` na API, `bun.lock` no web.
- Não existe manifest na raiz, workspace, nem qualquer `import` que atravesse a fronteira dos dois.
- **Validação:** o gate de tipos de cada app roda com o diretório do outro ausente e continua verde.

### 2. Quatro gates por app, nesta ordem

Análise estática e formato → tipos → build → testes. Cada um sai com código 0, **sem erro e sem aviso**.

| Gate | `apps/api` | `apps/web` |
| --- | --- | --- |
| Análise estática e formato | `eslint . --max-warnings 0` e `prettier --check` | `biome check .` |
| Tipos | `tsc --noEmit` | `tsc --noEmit` |
| Build | `nest build` | `vite build` |
| Testes | `jest` | `jest` |

- **O ESLint da API roda com regras type-aware**, o que exige `parserOptions.projectService` apontando
  para o `tsconfig.json`. Sem isso, `no-unsafe-assignment` não existe — e é a única regra que denuncia
  o `any` que escapa de decorator e de injeção.
- **`routeTree.gen.ts` é commitado e excluído do Biome.** É gerado pelo plugin do TanStack Router, mas a
  documentação oficial o trata como parte do runtime da aplicação, não como artefato de build.
- **A API não tem passo de geração antes do gate de tipos.** Entity do TypeORM é classe escrita à mão;
  não há client gerado, então `tsc --noEmit` roda sozinho.

### 3. A entrada de cada app só faz wiring

- `apps/api/src/main.ts` cria a aplicação a partir do `AppModule`, aplica o que é global e escuta.
  Não tem handler, não tem regra e não declara provider.
- `apps/api/src/app.module.ts` só importa: o `CoreModule` e os módulos de feature. Nada mais.
- O handler do health check vive em `apps/api/src/features/health/controller/health.controller.ts`,
  declarado pelo `health.module.ts`. Não tem service nem repository, porque não tem regra.
- **A documentação da API é o `@nestjs/swagger`**, montada no `main.ts` com `SwaggerModule.setup()`. O
  documento sai dos decorators das classes de DTO. O plugin de CLI do `@nestjs/swagger` fica ligado no
  `nest-cli.json`, para que o schema saia do tipo sem `@ApiProperty` repetido em cada campo.
- `apps/web/src/main.tsx` monta o React e o router. Não tem rota inline.
- **Verificação:** revisão do pull request, não cenário automatizado. É uma regra de organização de
  arquivos, sem resultado observável em runtime — o Cenário 1 prova que o health check responde, não
  onde ele mora.

### 4. Toda variável de ambiente é obrigatória e validada no boot

- **A API** lê o ambiente em um único módulo, `apps/api/src/core/config/`, com três arquivos:
  - `env.validation.ts` exporta a classe `Environment`, com um decorator de `class-validator` por
    variável, e `validateEnv(source: Record<string, unknown>): Environment` — **função pura, testada**.
    Ela converte com `plainToInstance`, valida com `validateSync` e, falhando, lança com a mensagem
    montada no formato abaixo;
  - `environment.service.ts` expõe o acessor tipado `get<Key extends keyof Environment>(key: Key)` sobre
    `ConfigService.getOrThrow`. **Nenhum outro arquivo chama `ConfigService.get`**, porque a sobrecarga
    que casa com `get('CHAVE')` devolve `any` e apaga a tipagem em silêncio;
  - `config.module.ts` registra `ConfigModule.forRoot({ validate: validateEnv })` e provê o
    `Environment`.

  **Nenhum arquivo do app lê `process.env`.** Quem lê é o `ConfigModule`; quem entrega valor é o
  `Environment` injetado.
- **O web** lê em um único módulo, `apps/web/src/shared/env/`, dividido em dois arquivos:
  - `env-schema.ts` exporta o schema Zod e `parseEnv(source: Record<string, unknown>)` — **função pura,
    testada**;
  - `env.ts` exporta `export const env = parseEnv(import.meta.env)` — **uma linha, sem teste**, porque
    `import.meta.env` é substituído em build-time pelo Vite e não existe sob o Jest.
- **Nenhum valor padrão em nenhum ambiente**, dev e teste incluídos.
- **Validação:** na criação do `AppModule`, antes de qualquer conexão ou listen — o `validate` do
  `ConfigModule` roda enquanto o container é construído, e o `TypeOrmModule` só conecta depois. O
  `main.ts` envolve o `NestFactory.create` e, capturando a falha de validação, escreve em `stderr` e
  sai com código `1`:

  > ```
  > Invalid environment:
  >   DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)
  > ```

  Uma linha `  <NOME>: <formato esperado>` por variável ausente ou inválida, em ordem alfabética, sob a
  linha fixa `Invalid environment:`. **O valor recebido nunca é impresso** — a mensagem descreve o
  formato esperado, nunca o conteúdo.

#### Variáveis desta entrega

| App | Nome | Tipo | Obrigatória | Formato esperado | Origem do valor |
| --- | --- | --- | --- | --- | --- |
| `api` | `DATABASE_URL` | string | Sim | `expected a PostgreSQL connection string (postgresql://…)` | dev: `.env` local a partir do `.env.example` · CI: serviço `postgres:18` do próprio job · homolog: #4 |
| `api` | `PORT` | inteiro | Sim | `expected an integer between 1 and 65535` | dev: `3333` · CI: `3333` · homolog: #4 |
| `api` | `WEB_ORIGIN` | string | Sim | `expected an absolute URL with no trailing slash (https://…)` | dev: `http://localhost:3000` · CI: `http://localhost:3000` · homolog: #4 |
| `web` | `VITE_API_URL` | string | Sim | `expected an absolute URL with no trailing slash (https://…)` | dev: `http://localhost:3333` · CI: não usada (o gate do web não sobe a API) · homolog: #4 |

`WEB_ORIGIN` nasce aqui, mas quem a consome é o CORS, na #3. Nesta entrega ela é validada e não usada.

`LOG_LEVEL` **não** nasce aqui: o logger é a #3, junto do `ExceptionFilter` global, e uma variável
validada sem consumidor nem destino é dívida, não preparação.

### 5. O Postgres de desenvolvimento sobe pelo compose da API

- `apps/api/compose.yaml`, Compose V2, carrega **somente** o Postgres 18 — a stack inteira é a #4.
- O serviço declara `healthcheck` com `pg_isready`, para que o compose da #4 possa depender dele.
- O volume de dados é nomeado, não um bind mount.

### 6. O `DataSource` do TypeORM é o único caminho até o banco

- `apps/api/src/core/db/data-source.ts` exporta o `DataSource` que a CLI do TypeORM usa para gerar e
  rodar migration. Ele lê a `DATABASE_URL` do ambiente do processo, porque a CLI roda fora do container
  de DI do Nest — é o **único** arquivo com essa licença, e ela existe por causa da CLI, não do app.
- `apps/api/src/core/db/db.module.ts` registra `TypeOrmModule.forRootAsync()` com a `DATABASE_URL` vinda
  do `Environment` injetado.
- **`synchronize` é `false` em todo ambiente, sem exceção.** Schema muda por migration versionada em
  `core/db/migrations/`, nunca por diff automático.
- **Nenhuma entity nesta entrega** — a primeira vem com a autenticação, na #3. O `entities` do
  `forRootAsync` aponta para o padrão de arquivo das features e resolve para lista vazia.
- Não há client gerado nem diretório de artefato: entity do TypeORM é classe escrita à mão.

### 7. O CI roda os gates em pull request para `develop` e para `main`

- Arquivo único: `.github/workflows/ci.yml`.
- Dois jobs, `api` e `web`, em paralelo. Cada um roda os quatro gates do seu app, em ordem, parando no
  primeiro que falhar.
- O job `api` sobe um serviço `postgres:18` com healthcheck `pg_isready`; as variáveis do job apontam
  para esse serviço efêmero.
- O job `api` usa **apenas `actions/setup-node` com Node 26**; o Bun não entra nele.
- O job `web` instala **Node 26 além do Bun**: o binário do Jest é `#!/usr/bin/env node` e não roda sem
  ele.
- Ambos instalam com lockfile congelado — `npm ci` na API, `bun install --frozen-lockfile` no web. Um
  lockfile desatualizado reprova o PR.
- **Um job por app, não um job por gate**: a quota do GitHub Actions cobra por job arredondado ao minuto
  inteiro, e este é um repositório privado.

### 8. Persistência e Auditoria

- **Tabelas/colunas alteradas:** nenhuma. Esta entrega não grava dado de negócio.
- **Auditoria:** `N/A` — não há ator autenticado nem acesso a prontuário. A trilha de auditoria é a
  issue #9.
- **Eventos/integrações disparados:** `N/A`.

---

## Erros

| Código | HTTP | Quando | Mensagem |
| --- | --- | --- | --- |
| — | — | — | — |

`N/A` para erros de API: `/health` não tem caminho de falha próprio, e o formato de erro da API
(`{ code, message, fields }`) nasce na #3 junto com o `onError` global e o primeiro `BusinessError`.

O único erro desta entrega é de **inicialização**, não de request: ambiente inválido, regra 4.

## Efeitos Colaterais

- **Persistência:** nenhuma.
- **Concorrência:** `N/A` — não há estado compartilhado nesta entrega.
- **Transação:** `N/A`.

---

## Cenários de Aceite (Gherkin)

### Cenário 1 — Health check responde (caminho feliz)

```gherkin
Dado que a API está inicializada com o ambiente válido
Quando é feita a requisição `GET /health`
Então o sistema responde `200`
E o corpo é exatamente `{"status":"ok"}`
E nenhuma consulta é feita ao banco
```

### Cenário 2 — Ambiente inválido derruba o boot da API (exceção)

```gherkin
Dado que a variável `DATABASE_URL` está ausente do ambiente
Quando a API é inicializada
Então o processo escreve em `stderr` a linha `Invalid environment:`
E escreve a linha `  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)`
E sai com código `1`
E não abre a porta HTTP
```

### Cenário 3 — Ambiente inválido é rejeitado no web (exceção)

```gherkin
Dado o schema de ambiente do web
Quando `parseEnv({ VITE_API_URL: "localhost:3333" })` é chamado
Então a chamada lança
E a mensagem contém `VITE_API_URL: expected an absolute URL with no trailing slash (https://…)`
E o valor recebido não aparece na mensagem
```

### Cenário 4 — O `DataSource` do TypeORM alcança o Postgres do compose (caminho feliz)

```gherkin
Dado o Postgres do `apps/api/compose.yaml` de pé
E a variável `DATABASE_URL` apontando para ele
Quando o `DataSource` é inicializado e abre uma transação vazia
Então a chamada completa sem lançar
E com o container parado a mesma chamada falha com `ECONNREFUSED`
```

**A transação vazia faz parte do cenário, não só o `initialize()`.** O critério é provar ida e volta
até o servidor, e a transação emite `BEGIN` e `COMMIT` de verdade. Um cenário que possa passar com a
dependência fora do ar não é critério de aceite. `dataSource.transaction(async () => {})` é API do
TypeORM, sem SQL cru e sem entity nenhuma declarada.

### Cenário 5 — A tela inicial do web renderiza com Tailwind aplicado (caminho feliz)

```gherkin
Dado que `apps/web` foi construído com `vite build`
Quando a rota `/` é aberta
Então a página exibe o texto `Clinicore` em um `h1`
E o `h1` está com as classes utilitárias do Tailwind aplicadas, não com o estilo padrão do navegador
```

### Cenário 6 — CI aprova um pull request com os gates verdes (caminho feliz)

```gherkin
Dado um pull request de uma branch `feature/*` para `develop`
Quando o workflow `CI` roda
Então o job `api` executa ESLint e Prettier, tipos, build e testes, todos com código 0
E o job `web` executa Biome, tipos, build e testes, todos com código 0
E o pull request aparece com o check verde
```

### Cenário 7 — CI reprova um gate vermelho (exceção)

```gherkin
Dado um pull request cujo código tem um erro de tipo em `apps/api`
Quando o workflow `CI` roda
Então o job `api` falha no gate de tipos
E os gates de build e de testes do job `api` não chegam a rodar
E o job `web` continua e conclui de forma independente
E o pull request aparece com o check vermelho
```

### Cenário 8 — Os apps não se importam (caminho alternativo)

```gherkin
Dado o diretório `apps/web` removido da árvore de trabalho
Quando os quatro gates de `apps/api` são executados
Então todos saem com código 0
E o mesmo vale na direção inversa, removendo `apps/api`
```

---

## Fora de Escopo

- **CORS e `credentials: "include"`** — issue #3. `WEB_ORIGIN` nasce aqui apenas validada.
- **`ExceptionFilter` global, `BusinessError` e o formato `{ code, message, fields }`** — issue #3. Nesta
  entrega não existe uma única rota que valide entrada ou que lance erro de negócio, então o filtro
  nasceria sem um caso testável.
- **Logger Pino, `LoggerService` e a variável `LOG_LEVEL`** — issue #3, pelo mesmo motivo: não há erro
  nem requisição com conteúdo para registrar.
- **Dockerfile dos apps, `compose.yaml` da raiz e deploy de homolog** — issue #4.
- **`worker.ts`, `@nestjs/bullmq` e `core/queue/`** — não há job nesta entrega; nascem com o primeiro,
  a issue #71.
- **Redis** — não há consumidor nesta entrega. Ele sobe no compose na issue #69, que é o primeiro uso:
  a contagem do limite por IP e a denylist de revogação de sessão.
- **`core/mail/` e Nodemailer** — idem, nasce com o primeiro e-mail.
- **Qualquer entity e a primeira migration** — issue #3.
- **Rota `(app)` protegida por sessão e layout da aplicação** — issue #3.
- **Git hooks, Storybook, matriz de versões no CI, cache de dependências no CI, badge de cobertura** —
  não pedidos por ninguém; entram quando houver dor medida.
- **Tema visual, tipografia e design system do web** — a rota `/` desta entrega existe só para provar que
  o build e o Tailwind funcionam.

## Quebra em Tasks

As tasks 1, 2 e 4 são refeitas na stack de #72 e viram sub-issues dela; as issues originais
(#57, #58, #60) ficam fechadas como histórico do que foi entregue na stack anterior. A task 3 é do
web, já entregue em #59, e **não muda**.

| # | Issue | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- | --- |
| 1 | sub-issue de #72 | Recreate apps/api on NestJS with green gates and a health check | `apps/api`: `package.json`, `tsconfig.json`, `nest-cli.json`, `eslint.config.mjs`, `.prettierrc`, `jest` no `package.json`, `src/main.ts`, `src/app.module.ts`, `src/core/config/` com `env.validation.ts`, `environment.service.ts` e `config.module.ts`, `src/features/health/` com módulo, controller e `__tests__` | Cenários 1 e 2 verdes; os quatro gates da API saem com código 0 | — |
| 2 | sub-issue de #72 | Add the development Postgres and the TypeORM DataSource to apps/api | `apps/api/compose.yaml`, `src/core/db/` com `data-source.ts`, `db.module.ts`, `migrations/` e `__tests__` | Cenário 4 verde; `tsc --noEmit` sai com código 0 | task 1 |
| 3 | #59 — entregue | Create apps/web with green gates and the root route | `apps/web`: inalterado. O web fica em Bun, Vite, Biome e TypeScript 7 até migrar para Next | Cenários 3 e 5 verdes; os quatro gates do web saem com código 0 | — |
| 4 | sub-issue de #72 | Update the CI workflow for the API toolchain | `.github/workflows/ci.yml`: job `api` com `setup-node` e `npm ci`, sem Bun; job `web` inalterado | Cenários 6, 7 e 8 verdes, observados em um pull request real | tasks 1 e 2 |

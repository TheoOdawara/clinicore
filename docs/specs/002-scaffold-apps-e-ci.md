# 002 — Criar os dois apps com gates verdes e CI

> **Status:** publicada
> **Perfil:** API
> **Módulo:** `apps/api`, `apps/web`, `.github/workflows`
> **Epic:** #1 — Plataforma
> **Issue:** #2

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

- `apps/api` e `apps/web` têm cada um o próprio `package.json` e o próprio `bun.lock`.
- Não existe manifest na raiz, workspace, nem qualquer `import` que atravesse a fronteira dos dois.
- **Validação:** o gate de tipos de cada app roda com o diretório do outro ausente e continua verde.

### 2. Quatro gates por app, nesta ordem

Análise estática e formato → tipos → build → testes. Cada um sai com código 0, **sem erro e sem aviso**.

| Gate | `apps/api` | `apps/web` |
| --- | --- | --- |
| Análise estática e formato | `biome check .` | `biome check .` |
| Tipos | `prisma generate && tsc --noEmit` | `tsc --noEmit` |
| Build | `bun build` | `vite build` |
| Testes | `bun test` | `jest` |

- **`prisma generate` faz parte do gate de tipos da API**, não é passo separado: no Prisma 7 ele deixou
  de rodar dentro de `migrate dev`, e sem ele o client não existe e o `tsc` falha.
- **`routeTree.gen.ts` é commitado e excluído do Biome.** É gerado pelo plugin do TanStack Router, mas a
  documentação oficial o trata como parte do runtime da aplicação, não como artefato de build.
- **O `bun test` da API roda com `CLAUDECODE` ausente do ambiente.** Com essa variável presente, a saída
  esconde os testes que passam.

### 3. A entrada de cada app só faz wiring

- `apps/api/src/server.ts` monta os plugins, registra os controllers e escuta. Não tem handler inline.
- O handler do health check vive em `apps/api/src/features/health/controller/health.controller.ts`. Não
  tem service nem repository, porque não tem regra.
- **A documentação da API é o `@elysiajs/openapi`**, montado no `server.ts` no modo baseado em schema: o
  documento sai em runtime dos schemas TypeBox declarados nas rotas. O `fromTypes` do plugin nunca é
  usado — ele depende da API programática do compilador, que o TypeScript 7.0 não tem. O
  `@elysiajs/swagger` parou na 1.3.1 e não tem versão para o Elysia 1.4: foi renomeado para
  `@elysiajs/openapi` na 1.3.
- `apps/web/src/main.tsx` monta o React e o router. Não tem rota inline.
- **Verificação:** revisão do pull request, não cenário automatizado. É uma regra de organização de
  arquivos, sem resultado observável em runtime — o Cenário 1 prova que o health check responde, não
  onde ele mora.

### 4. Toda variável de ambiente é obrigatória e validada no boot

- **A API** lê o ambiente em um único módulo, `apps/api/src/core/config/`, dividido em dois arquivos,
  com schema TypeBox:
  - `env-schema.ts` exporta o schema e `parseEnv(source: Record<string, string | undefined>)` — **função
    pura, testada**;
  - `env.ts` chama `parseEnv(Bun.env)`, escreve em `stderr` e sai com `1` quando a chamada lança, e
    exporta `env` — **uma responsabilidade, sem teste próprio**, coberta pelo Cenário 2.

  A divisão é a mesma do web e existe pela mesma razão: `env.ts` valida no import, e um teste de função
  pura não pode depender do ambiente do runner para carregar. **`env.ts` é o único arquivo do app que lê
  `Bun.env`**; nenhum outro lê `process.env`, `Bun.env` ou equivalente.
- **O web** lê em um único módulo, `apps/web/src/shared/env/`, dividido em dois arquivos:
  - `env-schema.ts` exporta o schema Zod e `parseEnv(source: Record<string, unknown>)` — **função pura,
    testada**;
  - `env.ts` exporta `export const env = parseEnv(import.meta.env)` — **uma linha, sem teste**, porque
    `import.meta.env` é substituído em build-time pelo Vite e não existe sob o Jest.
- **Nenhum valor padrão em nenhum ambiente**, dev e teste incluídos.
- **Validação:** na primeira linha do boot, antes de qualquer conexão ou listen. Falhando, o processo
  escreve em `stderr` e sai com código `1`:

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

### 5. O Postgres de desenvolvimento sobe pelo compose da API

- `apps/api/compose.yaml`, Compose V2, carrega **somente** o Postgres 18 — a stack inteira é a #4.
- O serviço declara `healthcheck` com `pg_isready`, para que o compose da #4 possa depender dele.
- O volume de dados é nomeado, não um bind mount.

### 6. O client do Prisma é o único caminho até o banco

- `apps/api/prisma/schema.prisma` tem `datasource` e `generator client` com `provider = "prisma-client"`
  e `output = "../generated/prisma"`. **Nenhum `model` nesta entrega** — a primeira migration vem com o
  Better Auth, na #3.
- `apps/api/src/core/db/prisma.ts` instancia
  `new PrismaClient({ adapter: new PrismaPg({ connectionString: env.DATABASE_URL }) })`.
  `PrismaPg` vem de `@prisma/adapter-pg` e recebe um objeto de configuração, não uma string solta.
  Sem `previewFeatures`: driver adapters são GA e obrigatórios no Prisma 7.
- `apps/api/generated/` é ignorado pelo git e pelo Biome, e incluído no `tsc`.

### 7. O CI roda os gates em pull request para `develop` e para `main`

- Arquivo único: `.github/workflows/ci.yml`.
- Dois jobs, `api` e `web`, em paralelo. Cada um roda os quatro gates do seu app, em ordem, parando no
  primeiro que falhar.
- O job `api` sobe um serviço `postgres:18` com healthcheck `pg_isready`; as variáveis do job apontam
  para esse serviço efêmero.
- O job `web` instala **Node 26 além do Bun**: o binário do Jest é `#!/usr/bin/env node` e não roda sem
  ele.
- Ambos instalam com lockfile congelado — um lockfile desatualizado reprova o PR.
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

### Cenário 4 — O client do Prisma alcança o Postgres do compose (caminho feliz)

```gherkin
Dado o Postgres do `apps/api/compose.yaml` de pé
E a variável `DATABASE_URL` apontando para ele
Quando o client do Prisma abre uma transação vazia
Então a chamada completa sem lançar
E com o container parado a mesma chamada falha com `ECONNREFUSED`
```

**Não é `$connect()`.** Com driver adapter o `$connect()` não abre conexão: ele resolve igual com o
Postgres desligado, medido nesta stack. Um cenário que passa com a dependência fora do ar não é
critério de aceite. A transação vazia é API do Prisma — sem raw SQL e sem `model` — e é o menor
caminho que faz ida e volta de verdade até o servidor.

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
Então o job `api` executa Biome, tipos, build e testes, todos com código 0
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

- **CORS, `trustedOrigins` e `credentials: "include"`** — issue #3, junto com o Better Auth. `WEB_ORIGIN`
  nasce aqui apenas validada.
- **`onError` global, `BusinessError` e o formato `{ code, message, fields }`** — issue #3. Nesta entrega
  não existe uma única rota que valide entrada ou que lance erro de negócio, então o tratador nasceria
  sem um caso testável.
- **Dockerfile dos apps, `compose.yaml` da raiz e deploy de homolog** — issue #4.
- **`worker.ts`, pg-boss e `core/queue/`** — não há job nesta entrega; nasce com a primeira feature que
  enfileira.
- **`core/mail/` e Nodemailer** — idem, nasce com o primeiro e-mail.
- **Qualquer `model` no `schema.prisma` e a primeira migration** — issue #3.
- **Rota `(app)` protegida por sessão e layout da aplicação** — issue #3.
- **Git hooks, Storybook, matriz de versões no CI, cache de dependências no CI, badge de cobertura** —
  não pedidos por ninguém; entram quando houver dor medida.
- **Tema visual, tipografia e design system do web** — a rota `/` desta entrega existe só para provar que
  o build e o Tailwind funcionam.

## Quebra em Tasks

| # | Issue | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- | --- |
| 1 | #57 | Create apps/api with green gates and a health check | `apps/api`: `package.json`, `tsconfig.json`, `biome.json`, `src/server.ts`, `src/core/config/` com `env-schema.ts` e `env.ts`, `src/features/health/` com controller e `__tests__` | Cenários 1 e 2 verdes; os quatro gates da API saem com código 0 | — |
| 2 | #58 | Add the development Postgres and the Prisma client to apps/api | `apps/api/compose.yaml`, `prisma/schema.prisma`, `src/core/db/prisma.ts` e `src/core/db/__tests__` | Cenário 4 verde; `prisma generate` e `tsc --noEmit` saem com código 0 | #57 |
| 3 | #59 | Create apps/web with green gates and the root route | `apps/web`: `package.json`, `vite.config.ts`, `tsconfig.json`, `biome.json`, `jest.config.ts`, `src/main.tsx`, `src/routes/`, `src/styles/`, `src/shared/env/` com `__tests__` | Cenários 3 e 5 verdes; os quatro gates do web saem com código 0 | — |
| 4 | #60 | Add the CI workflow for pull requests to develop and main | `.github/workflows/ci.yml` | Cenários 6, 7 e 8 verdes, observados em um pull request real | #57, #58, #59 |

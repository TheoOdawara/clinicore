# 0001. API em NestJS, TypeORM e Redis

- Status: accepted
- Date: 2026-09-19

## Context

A API tinha 10 arquivos e nada em produção quando esta decisão foi tomada. Quatro dos arquivos
da branch `feature/65-auth-env-logger-error-handler` existiam só porque o Elysia não traz o
equivalente: `core/logger/request-logger.ts`, `core/errors/error-handler.ts`, `core/shutdown.ts`
e `core/config/env-*.ts`, este último com 106 linhas de regex à mão porque o TypeBox é validador
de DTO e não tem formato pronto.

As camadas descritas em `CLAUDE.md` — `Controller → Service → Repository → Prisma` — são
convenção escrita à mão, sem nada que impeça um service de importar o client do banco.

A regra escolhida para a troca: **tudo que o framework traz nativo, mais o que a documentação
dele recomenda.** Uma lacuna de infraestrutura transversal é território de framework; uma lacuna
de componente não é.

O `apps/web` está fora desta decisão.

## Decision

O `apps/api` é reconstruído em **NestJS 11**.

| Sai | Entra |
| --- | --- |
| Elysia 1.4 + TypeBox | NestJS 11 + `class-validator` / `class-transformer` |
| Prisma 7.10 | TypeORM 1.1 com `@nestjs/typeorm` e `pg` |
| Better Auth | `@nestjs/passport`, `@nestjs/jwt`, `@node-rs/argon2` |
| pg-boss no próprio Postgres | `@nestjs/bullmq` com Redis |
| `@elysiajs/openapi` | `@nestjs/swagger` |
| `fetch` solto para terceiro | `@nestjs/axios`, com config e interceptor por integração |
| `core/logger`, `core/errors`, `core/shutdown` | `LoggerService` com Pino, `ExceptionFilter` global, `enableShutdownHooks()` |
| `core/config/env-*.ts` em TypeBox | `@nestjs/config` validado por `class-validator` |
| Bun 1.4 como runtime e gerenciador | Node 26 e npm |
| Biome 2.5 | ESLint, `typescript-eslint` e Prettier |
| `bun test` | Jest, supertest e `@nestjs/testing` |
| TypeScript 7.0 | TypeScript 6.0 |

Decisões de detalhe dentro da regra:

- **Um validador só na API: `class-validator` e `class-transformer`**, para DTO e para
  ambiente. Eles já são peer dependency declarada do `@nestjs/common`, então entram instalados
  de qualquer forma.
  **Zod fica reservado para a migração ao Nest 12**, que é o gatilho: só a partir do 12 existe
  `ArgumentMetadata.schema` e a sobrecarga `Body(options)`, que permitem usar Zod também no DTO
  e na resposta. Trocar antes disso significaria Zod no ambiente e `class-validator` no DTO, que
  é o oposto de um validador só.
- **Pino atrás de um `LoggerService` nativo** (`app.useLogger()`), sem `nestjs-pino`. A
  documentação recomenda o Pino; o wrapper é um terceiro a mais, e plugin de log de terceiro já
  estava vetado no repo.
- **Nest 11 e não Nest 12.** O `@nestjs/common@12` publica `"type": "module"`, e um Jest em
  CommonJS não carrega ESM do `node_modules` sem `--experimental-vm-modules`.
- Os pacotes-satélite saltaram a numeração mas declaram `@nestjs/common: ^11.0.0 || ^12.0.0`:
  `@nestjs/config@12`, `@nestjs/schedule@12`, `@nestjs/event-emitter@12` e `@nestjs/throttler@6.7`
  funcionam sobre o Nest 11.

Esta decisão **emenda a seção Architecture do `CLAUDE.md` do repo**, que é reescrita junto.

## Consequences

Fica mais fácil:

- Três arquivos de `core/` deixam de existir: o log de requisição vira interceptor, o tratamento
  de erro vira `ExceptionFilter` global e o desligamento vira `enableShutdownHooks()`.
- As camadas param de ser convenção: o container de DI é quem decide quem alcança o quê.
- OpenAPI, rate limit, health check, agendamento e eventos passam a ser pacotes `@nestjs/*`.

Fica mais difícil, e é aceito:

- **Redis vira dependência de infraestrutura** em desenvolvimento, homolog e produção. Revoga a
  linha "sem Redis" do `CLAUDE.md`.
- **O que o Better Auth dava pronto passa a ser código escrito.** Verificação de e-mail, reset de
  senha, refresh token, limite de 5 sessões por usuário e vínculo da conta Google. As issues #65
  a #71 crescem em esforço sem mudar de assunto.
- **A API cai para TypeScript 6.0.** O `@nestjs/cli@11.0.24` carrega `typescript 5.9.3` interno e
  o ferramental do Nest depende da API programática do compilador, que o TypeScript 7.0 não tem.
  O `apps/web` segue no 7.0; os apps são independentes.
- **O repo passa a ter dois gerenciadores de pacote** — npm na API, Bun no web — até o web
  migrar. Estado transitório, declarado no `CLAUDE.md`.
- **`docs/specs/autenticacao/003-autenticacao-web.md` segue descrevendo o cliente do Better
  Auth.** É resolvida quando o front migrar, não aqui.
- O Jest da API roda em CommonJS por causa do Nest 11, enquanto o do web roda em ESM.
- **O `class-validator` não tem decorador de CIDR**, que o `TRUSTED_PROXIES` precisa. O
  `validator` 13, que ele já traz como dependência, tem `isIPRange` — o decorador sai de um
  `registerDecorator` de poucas linhas, não de regex à mão.

## Alternatives considered

- **Ficar no Elysia e trocar só o validador de ambiente** (envalid ou Zod) · rejeitada: resolve as
  106 linhas de regex e deixa logger, tratamento de erro, desligamento e as camadas por convenção
  escritos à mão.
- **NestJS 12** · rejeitada: é ESM-only e obriga `--experimental-vm-modules` no Jest. O
  `vm.SourceTextModule` continua Stability 1 na documentação do Node 26, o roadmap de
  estabilização (`nodejs/node#37648`) está aberto desde 2021 e a proposta de redesenho
  (`nodejs/node#62720`, de 2026-04-13) é rascunho sem release alvo. Esperar não tem data.
- **Manter o Prisma com um `PrismaModule` escrito à mão** · rejeitada: não existe `@nestjs/prisma`
  oficial, e a regra escolhida foi tomar o que o framework padroniza.
- **Manter o pg-boss no Postgres para não subir Redis** · rejeitada: a fila oficialmente suportada
  (`@nestjs/bullmq` e `@nestjs/bull`) exige Redis.
- **Manter o Better Auth ao lado do Nest** · rejeitada: mesma regra, e não há adapter oficial.
- **Migrar o web para Next no mesmo movimento** · rejeitada: a lacuna do front é componente, não
  infraestrutura transversal. Decidida em separado, com gatilho próprio.
- **Zod para o ambiente, `class-validator` para os DTOs** · rejeitada por ora: o
  `@nestjs/config@12` aceita Standard Schema e o Zod resolveria formato de ambiente melhor, mas
  no Nest 11 ele não alcança o DTO, e duas bibliotecas de schema por causa de um único arquivo
  de configuração não se paga. Reabre na migração ao Nest 12.
- **NestJS sobre Fastify em vez de Express** · rejeitada: o `@nestjs/passport` depende do
  Express. Passport e o middleware em volta chamam `res.setHeader()` e `res.end()`, que o
  Fastify não expõe no reply, e a serialização de sessão com `req.logIn()` é onde quebra — ou
  seja, exatamente o login pelo Google da issue #68. Com JWT puro funcionaria; a saída
  conhecida é remendar o reply do Fastify no `main.ts` para imitar o Express. A própria
  documentação do Nest avisa que "each recipe that relies on Express may no longer work".
  Reabre se o Passport sair da stack.
- **`nestjs-pino`** · rejeitada: a documentação recomenda o Pino, não o wrapper.
- **Manter o Biome na API** · rejeitada: não faz lint type-aware, e `no-unsafe-assignment` é a
  única regra que denuncia o `ConfigService.get` devolvendo `any`.

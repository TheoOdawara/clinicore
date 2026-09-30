# Clinicore — contrato do repositório

Aditivo ao contrato global (`~/.claude/CLAUDE.md`); em conflito, **este arquivo vence**. Nada aqui
repete o que já é global.

O produto — problema, escopo, go-live da clínica piloto e critérios de aceite — está em
`docs/requirements.md`. Este arquivo diz **como** o sistema é construído, nunca **o que** ele faz.

## Stack

Quatro apps **independentes** em `apps/`, cada um com o próprio manifesto e o próprio lockfile. Sem
manifest na raiz, sem workspaces, sem código ou tipo compartilhado por import. A fronteira entre eles é
o contrato HTTP.

**`apps/api`, `apps/web` e `apps/site` usam npm sobre Node 26.** É estado decidido, não transitório.

**`apps/api-rs` é a API em Rust, com axum e sqlx, e convive com o `apps/api` congelado e sem job no
CI até a #121.** A stack, os gates e as camadas dele estão no `apps/api-rs/CLAUDE.md`.

**`apps/mobile` é o app nativo iOS e Android em Flutter**, com paridade de telas com o web, offline com
fila de escrita e cliente HTTP gerado do OpenAPI da API — gerar do contrato não é importar código.
Decisão em `docs/decisions/0007-app-nativo-em-flutter-com-offline.md`.

Cada app tem o seu domínio: `clinicore.com.br` é a landing pública, `app.clinicore.com.br` é o sistema e
`api.clinicore.com.br` é a API. Os três compartilham o mesmo domínio registrável, e é isso que mantém o
cookie de sessão em `SameSite=Lax`.

| | `apps/api` | `apps/web` | `apps/site` | `apps/mobile` |
|---|---|---|---|---|
| Papel | o contrato HTTP | sistema da clínica, PWA instalável, só usuário autenticado | landing pública e indexável | app nativo iOS e Android, só usuário autenticado |
| Runtime e pacotes | Node 26 · npm | Node 26 · npm | Node 26 · npm | Flutter 3.47 · pub |
| Tipos | TypeScript 6.0 | TypeScript 6.0 | TypeScript 6.0 | Dart 3.13, com `strict-casts`, `strict-inference` e `strict-raw-types` |
| Lint e formato | ESLint 10 · `typescript-eslint` 8, com regras type-aware · Prettier 3 | ESLint 9 · `typescript-eslint` 8, com regras type-aware · Prettier 3 | ESLint 9 · `typescript-eslint` 8, com regras type-aware · `eslint-config-next` 16.3 · Prettier 3 | `flutter_lints` 6 · `dart format` |
| Framework | NestJS 11 sobre Express, validação e DTO em `class-validator` e `class-transformer`, OpenAPI por `@nestjs/swagger`, health check por `@nestjs/terminus` | React 19.3 · Vite 8.3 · TanStack Router 1.170 (file-based) | React 19.3 · Next.js 16.3 com App Router e `output: "standalone"` · `next/image` e `next/font` | Flutter 3.47 · `go_router` 18.0 |
| Dados | PostgreSQL 18 · TypeORM 1.1 com `@nestjs/typeorm` e `pg` | TanStack Query 5.102 · axios 1 · Zod 4.6 | — | cliente Retrofit sobre `dio` 5.11, gerado por `swagger_parser` 1.44 com `json_serializable` |
| Formulário | — | TanStack Form 1.33 com schema Zod | — | — |
| Estilo | — | Tailwind 4.3 · shadcn/ui sobre Radix e CVA · `tw-animate-css` | Tailwind 4.3 · shadcn/ui, com cópia própria | — |
| Auth | `@nestjs/passport`, `@nestjs/jwt` e `@node-rs/argon2`; access token curto e refresh na tabela `session` | axios contra `/users` e `/sessions` da API, com o refresh no interceptor | — | `dio` com `Clinicore-Client: mobile` e o token no `Authorization` |
| Fila e agendamento | `@nestjs/bullmq` · `@nestjs/schedule` | — | — | — |
| Redis 8 | fila, contagem do limite por IP e denylist de revogação de sessão | — | — | — |
| HTTP de saída | `@nestjs/axios` sobre axios 1 | — | — | — |
| Configuração | `@nestjs/config`, validada por `class-validator` no boot | — | — | `--dart-define-from-file`, lida e validada em `lib/shared/env/env.dart` |
| E-mail | Nodemailer pelo SMTP do Gmail (`smtp.gmail.com:587`), em todos os ambientes | — | — | — |
| Log | Pino 10 atrás de um `LoggerService` do Nest, JSON em stdout, com `redact` | — | — | — |
| Testes | Jest 30 · `ts-jest` · supertest · `@nestjs/testing` | Jest 30 · `@swc/jest` · jsdom · Testing Library | — | `flutter_test` |

As decisões que trouxeram esta stack, o que foi descartado e por quê:
`docs/decisions/0001-api-em-nestjs-typeorm-e-redis.md`,
`docs/decisions/0002-web-e-site-em-vite.md`,
`docs/decisions/0003-design-system-com-shadcn-ui-e-react-bits.md`,
`docs/decisions/0004-lint-do-front-em-eslint.md`,
`docs/decisions/0005-front-em-npm-sobre-node.md`,
`docs/decisions/0006-api-rest-e-problem-details.md`,
`docs/decisions/0007-app-nativo-em-flutter-com-offline.md` e
`docs/decisions/0008-site-em-next-standalone.md`.

## Comandos

Cada comando roda de dentro do diretório do seu app.

| Gate | `apps/api` | `apps/web` | `apps/site` | `apps/mobile` |
|---|---|---|---|---|
| Análise estática e formato | `npm run lint` · `npm run format:check` | `npm run lint` · `npm run format:check` | `npm run lint` · `npm run format:check` | `flutter analyze --fatal-infos` · `dart format --output=none --set-exit-if-changed .` |
| Tipos | `npm run typecheck` | `npm run typecheck` | `npm run typecheck` | dentro do `flutter analyze` |
| Build | `npm run build` | `npm run build` | `npm run build` | — fora do CI até a publicação |
| Testes | `npm run test` | `npm run test` | — enquanto não houver lógica a testar | `flutter test` |
| Testes e2e | `npm run test:e2e` | — | — | — |

Instalação: `npm ci` nos três apps Node e `flutter pub get --enforce-lockfile` no `apps/mobile`.

**Os valores de desenvolvimento moram no Infisical**, no projeto `Clinicore` que o `.infisical.json`
da raiz liga ao repo inteiro, com uma pasta por app (`/api` para o `apps/api-rs`). Comando que precisa
de variável roda dentro de `infisical run --path=/<pasta> --`, e o `CLAUDE.md` de cada app diz quais.

O gate de tipos do web exige o `src/routeTree.gen.ts`, gerado pelo plugin do TanStack Router.
Ele é commitado, então só um `src/routes/` alterado sem `vite build` ou `vite dev` desde a alteração
deixa o `tsc` olhando para uma árvore velha.

**O build do site tem um passo a mais que o `npm run build`:** o job confere que o
`.next/server/app/index.html` tem conteúdo. É o que denuncia uma página que o prerender gerou vazia.

## Arquitetura

```
apps/
├── api/                     NestJS · :3333 · congelado até a #121
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
├── api-rs/                  Rust · axum · sqlx · :3333 · crates core e app
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
├── site/                    Next.js standalone · :4321 · clinicore.com.br
│   └── src/
│       ├── app/             App Router: a pasta é a URL, layout.tsx e page.tsx por rota
│       ├── sections/        blocos da landing: hero, preços, dúvidas
│       ├── components/      UI base do shadcn e o que for copiado
│       ├── assets/          imagem e vídeo da landing
│       └── styles/
└── mobile/                  Flutter · iOS e Android · br.com.clinicore.dev
    ├── config/              example.json commitado; <ambiente>.json fora do git
    ├── android/ ios/        projeto nativo
    ├── test/                espelha lib/
    └── lib/
        ├── main.dart        só boot
        ├── app/             MaterialApp e o GoRouter
        ├── features/<feature>/
        └── shared/          env, http e api — o cliente gerado, nunca editado à mão
compose.yaml                 stack inteira: site, web, api, worker, Postgres e Redis
docs/
```

**O browser fala direto com a API, em origem cruzada.** Em dev são as portas :3000 e :3333; em homolog e
produção, `app.clinicore.com.br` e `api.clinicore.com.br`. Por isso o `enableCors()` da API libera as
origens de `ALLOWED_ORIGINS` com `credentials: true`, e o web chama com `withCredentials: true`.

**A regra de cada app mora no `CLAUDE.md` dele** — `apps/api/CLAUDE.md`, `apps/api-rs/CLAUDE.md`, `apps/web/CLAUDE.md`,
`apps/site/CLAUDE.md` e `apps/mobile/CLAUDE.md` carregam as camadas, a forma da URL e as pegadinhas da stack daquele app,
e só entram em contexto quando o trabalho toca a pasta.

**`apps/site` tem servidor, mas não tem sessão.** O processo Node do standalone serve a landing e
otimiza a imagem dela; ele não participa de sessão, não lê cookie e não recebe segredo. Se um dia
chamar a API — formulário de contato, pedido de demonstração, código de convite —, é por rota pública e
a origem dele entra em `ALLOWED_ORIGINS`. Redirecionamento de autenticação nunca aponta para ele.

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
inglês. O chat segue em pt-BR. **A URL é a única coisa que depende do app**, e o `CLAUDE.md` de
cada app diz a sua.

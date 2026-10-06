# Clinicore — contrato do repositório

Aditivo ao contrato global (`~/.claude/CLAUDE.md`); em conflito, **este arquivo vence**. Nada aqui
repete o que já é global.

O produto — problema, escopo, go-live da clínica piloto e critérios de aceite — está em
`docs/requirements/`. Este arquivo diz **como** o sistema é construído, nunca **o que** ele faz. O mapa
do sistema está em `docs/architecture/`, o modelo de dados em `docs/data-model/`, e o `docs/README.md` é o
índice de tudo.

## Stack

Quatro apps **independentes** em `apps/`, cada um com o próprio manifesto e o próprio lockfile. Sem
manifest na raiz, sem workspaces, sem código ou tipo compartilhado por import. A fronteira entre eles é
o contrato HTTP.

**`apps/web` e `apps/site` usam npm sobre Node 26.** É estado decidido, não transitório.

**`apps/api` é a API em Rust, com axum e sqlx.** O NestJS que ela foi está na tag `api-nestjs-final`.
As camadas, os gates completos e as pegadinhas estão no `apps/api/AGENTS.md`.

**`apps/mobile` é o app nativo iOS e Android em Flutter**, com paridade de telas com o web, offline com
fila de escrita e cliente HTTP gerado do OpenAPI da API — gerar do contrato não é importar código.
Decisão em `docs/decisions/0007-app-nativo-em-flutter-com-offline.md`.

Cada app tem o seu domínio: `clinicore.com.br` é a landing pública, `app.clinicore.com.br` é o sistema e
`api.clinicore.com.br` é a API. Os três compartilham o mesmo domínio registrável, e é isso que mantém o
cookie de sessão em `SameSite=Lax`.

| | `apps/api` | `apps/web` | `apps/site` | `apps/mobile` |
|---|---|---|---|---|
| Papel | o contrato HTTP | sistema da clínica, PWA instalável, só usuário autenticado | landing pública e indexável | app nativo iOS e Android, só usuário autenticado |
| Runtime e pacotes | Rust 1.98.1, edition 2024 · cargo | Node 26 · npm | Node 26 · npm | Flutter 3.47 · pub |
| Tipos | Rust, com `unsafe_code = "forbid"` | TypeScript 6.0 | TypeScript 6.0 | Dart 3.13, com `strict-casts`, `strict-inference` e `strict-raw-types` |
| Lint e formato | `cargo clippy` com `-D warnings` · `cargo fmt` | ESLint 9 · `typescript-eslint` 8, com regras type-aware · Prettier 3 | ESLint 9 · `typescript-eslint` 8, com regras type-aware · `eslint-config-next` 16.3 · Prettier 3 | `flutter_lints` 6 · `dart format` |
| Framework | axum 0.8 sobre tokio e tower, validação por `validator`, OpenAPI por `utoipa` | React 19.3 · Vite 8.3 · TanStack Router 1.170 (file-based) | React 19.3 · Next.js 16.3 com App Router e `output: "standalone"` · `next/image` e `next/font` | Flutter 3.47 · `go_router` 18.0 |
| Dados | PostgreSQL 18 · sqlx 0.9, com as consultas conferidas contra o schema na compilação | TanStack Query 5.102 · axios 1 · Zod 4.6 | — | cliente Retrofit sobre `dio` 5.11, gerado por `swagger_parser` 1.44 com `json_serializable` |
| Formulário | — | TanStack Form 1.33 com schema Zod | — | — |
| Estilo | — | Tailwind 4.3 · shadcn/ui sobre Radix e CVA · `tw-animate-css` | Tailwind 4.3 · shadcn/ui, com cópia própria | — |
| Auth | `jsonwebtoken` e `argon2`; access token curto e refresh na tabela `sessions`; login com Google por `oauth2`, com PKCE | axios contra `/users` e `/sessions` da API, com o refresh no interceptor | — | `dio` com `Clinicore-Client: mobile` e o token no `Authorization` |
| Fila e agendamento | limpeza periódica dentro do processo da API; o worker nasce com o primeiro job da fila | — | — | — |
| Redis 8 | contagem do limite por IP e denylist de revogação de sessão | — | — | — |
| HTTP de saída | `oauth2` 5.0 sobre `reqwest` 0.12, só para o Google | — | — | — |
| Configuração | variáveis de ambiente tipadas e validadas no boot, em `crates/core` | — | — | `--dart-define-from-file`, lida e validada em `lib/shared/env/env.dart` |
| E-mail | `lettre` por SMTP, com o layout em `askama` | — | — | — |
| Log | `tracing` com `tracing-subscriber`, JSON em stdout | — | — | — |
| Testes | `cargo test`, com os testes de integração em `crates/app/tests/` | Jest 30 · `@swc/jest` · jsdom · Testing Library | — | `flutter_test` |

As decisões que trouxeram esta stack, o que foi descartado e por quê:
`docs/decisions/0001-api-em-nestjs-typeorm-e-redis.md`,
`docs/decisions/0002-web-e-site-em-vite.md`,
`docs/decisions/0003-design-system-com-shadcn-ui-e-react-bits.md`,
`docs/decisions/0004-lint-do-front-em-eslint.md`,
`docs/decisions/0005-front-em-npm-sobre-node.md`,
`docs/decisions/0006-api-rest-e-problem-details.md`,
`docs/decisions/0007-app-nativo-em-flutter-com-offline.md`,
`docs/decisions/0008-site-em-next-standalone.md`,
`docs/decisions/0009-api-em-rust-com-axum-e-sqlx.md`, que substitui a 0001,
`docs/decisions/0010-api-rs-com-crate-por-processo.md`,
`docs/decisions/0011-openapi-da-api-rs-nasce-com-a-rota.md`,
`docs/decisions/0012-revogacao-de-sessao-antes-de-apagar.md` e
`docs/decisions/0013-assinatura-digital-em-duas-camadas.md`.

## Linguagem e scripts

O código de aplicação é Rust no `apps/api`, TypeScript no `apps/web` e no `apps/site`, e Dart no
`apps/mobile`. **O repo não tem nenhum script hoje**, e a linguagem do primeiro está em aberto: ela é
decidida quando ele for necessário, e registrada aqui.

## Comandos

Cada comando roda de dentro do diretório do seu app.

| Gate | `apps/api` | `apps/web` | `apps/site` | `apps/mobile` |
|---|---|---|---|---|
| Análise estática e formato | `cargo clippy --all-targets -- -D warnings` · `cargo fmt --check` | `npm run lint` · `npm run format:check` | `npm run lint` · `npm run format:check` | `flutter analyze --fatal-infos` · `dart format --output=none --set-exit-if-changed .` |
| Tipos | dentro do `cargo clippy` | `npm run typecheck` | `npm run typecheck` | dentro do `flutter analyze` |
| Build | `cargo build` | `npm run build` | `npm run build` | — fora do CI até a publicação |
| Testes | `cargo test` | `npm run test` | — enquanto não houver lógica a testar | `flutter test` |
| Testes e2e | — dentro do `cargo test` | — | — | — |

Instalação: `npm ci` nos dois apps Node e `flutter pub get --enforce-lockfile` no `apps/mobile`; o
`apps/api` não tem passo de instalação, o `cargo` baixa as dependências no primeiro build. Os comandos
da API que tocam o banco rodam sob `infisical run --path=/api --`, como diz o `apps/api/AGENTS.md`.

O gate de tipos do web exige o `src/routeTree.gen.ts`, gerado pelo plugin do TanStack Router.
Ele é commitado, então só um `src/routes/` alterado sem `vite build` ou `vite dev` desde a alteração
deixa o `tsc` olhando para uma árvore velha.

**O build do site tem um passo a mais que o `npm run build`:** o job confere que o
`.next/server/app/index.html` tem conteúdo. É o que denuncia uma página que o prerender gerou vazia.

## Arquitetura

```
apps/
├── api/                     Rust · axum · sqlx · :3333
│   ├── compose.yaml         Postgres e Redis de desenvolvimento
│   ├── migrations/          SQL aplicado pelo sqlx
│   └── crates/
│       ├── core/            configuração, Postgres, Redis e e-mail
│       └── app/             a API inteira; gera o binário `api`
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
docs/
```

**O `compose.yaml` da raiz, com a stack inteira, ainda não existe.** Não há hospedagem hoje: o
deploy planejado é uma VPS com Coolify, e o compose da raiz nasce com ele. O único compose é o
`apps/api/compose.yaml`, com o Postgres e o Redis de desenvolvimento.

**O browser fala direto com a API, em origem cruzada.** Em dev são as portas :3000 e :3333; em homolog e
produção, `app.clinicore.com.br` e `api.clinicore.com.br`. Por isso o CORS da API libera as
origens de `ALLOWED_ORIGINS` com `credentials: true`, e o web chama com `withCredentials: true`.

**A regra de cada app mora no `AGENTS.md` dele** — `apps/api/AGENTS.md`, `apps/web/AGENTS.md`,
`apps/site/AGENTS.md` e `apps/mobile/AGENTS.md` carregam as camadas, a forma da URL e as pegadinhas da stack daquele app,
e só entram em contexto quando o trabalho toca a pasta.

**`apps/site` tem servidor, mas não tem sessão.** O processo Node do standalone serve a landing e
otimiza a imagem dela; ele não participa de sessão, não lê cookie e não recebe segredo. Se um dia
chamar a API — formulário de contato, pedido de demonstração, código de convite —, é por rota pública e
a origem dele entra em `ALLOWED_ORIGINS`. Redirecionamento de autenticação nunca aponta para ele.

## Branches

- **`main` é produção e `develop` é homolog, as duas protegidas por convenção** — nunca commitar
  direto nelas. O GitHub não aplica a proteção: nenhuma das duas tem regra de proteção nem ruleset
  configurado.
- Cada entrega nasce em `feature/<número>-<assunto>` a partir de `develop` e volta para ela por pull
  request. Correção urgente é `hotfix/<número>-<assunto>`, a partir da `main`.
- Release é pull request de `develop` para `main`.
- **A mensagem de commit é só o título.** O porquê e o detalhe vão na descrição do pull request.
- **CI roda em pull request para `develop` e para `main`**, por `.github/workflows/ci.yml`, com um job por app.

## Backlog

- **O backlog é o GitHub Project `Clinicore`**, em `https://github.com/users/TheoOdawara/projects/3`, com a
  sprint de duas semanas no campo `Sprint`.
- **A branch padrão do GitHub é a `main`**, então a palavra de fechamento só fecha a issue quando o
  release chega nela. O pull request para a `develop` não liga a issue nem move o item.
- **`In Review` e `Staging` são marcados à mão**, com `gh project item-edit`: `In Review` ao abrir o
  pull request, `Staging` no merge na `develop`. O `Done` vem sozinho quando a issue fecha.
- **A ordem de entrega é API e banco, depois web e PWA, depois mobile.** A próxima issue é a primeira
  aberta nessa ordem, respeitando o `Depende de` de cada uma.
- **Toda issue de tela ou de comportamento do `apps/web` tem uma gêmea no `apps/mobile`**, criada junto
  com ela: é o que sustenta a paridade da ADR 0007.
- **Toda issue leva a label do tipo do commit** (`feat`, `fix`, `refactor`, `docs`, `chore`, `ci`, `test`,
  `build`) **e a `area:<app>`** do que ela toca: `area:api`, `area:web`, `area:site`, `area:mobile` ou
  `area:ci`.

## Idioma

**Só a documentação é pt-BR.** Código, identificadores, pastas, arquivos e mensagens de commit são
inglês. O chat segue em pt-BR. **A URL é a única coisa que depende do app**, e o `AGENTS.md` de
cada app diz a sua.

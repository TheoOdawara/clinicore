# Clinicore

SaaS de gestão para clínicas odontológicas: agenda, prontuário clínico e estético, financeiro, estoque e
laboratório num só lugar, multi-tenant, para redes de clínicas e dentistas autônomos.

> **Em desenvolvimento.** A API já tem a autenticação completa; os módulos de produto vêm em seguida.

## Stack

| App | Papel | Tecnologia |
|---|---|---|
| `apps/api` | contrato HTTP | NestJS 11 · TypeScript · PostgreSQL 18 com TypeORM · Redis 8 · Passport/JWT · Pino · Swagger |
| `apps/web` | sistema da clínica (PWA) | React 19 · Vite · TanStack Router/Query/Form · Zod · Tailwind + shadcn/ui |
| `apps/site` | landing pública | Next.js 16 standalone · React 19 |
| `apps/mobile` | app iOS e Android | Flutter · Dio · cliente HTTP gerado do OpenAPI da API |

Os apps são independentes: cada um tem o próprio manifesto e lockfile, e a fronteira entre eles é o contrato HTTP.

## O que a API já faz

- Cadastro, login e sessão: cookie no web, token no app mobile; access token curto e refresh token rotacionado, guardado na tabela `session`
- Login com Google, vinculado à conta existente pelo e-mail verificado
- Troca de senha do usuário autenticado
- Reuso de refresh token derruba a sessão; revogação imediata por denylist no Redis
- Limite de cinco sessões ativas por usuário, com lock de linha contra logins concorrentes
- Verificação de e-mail e redefinição de senha, com limite de envio por endereço
- Rate limit das rotas de autenticação por IP de cliente confiável
- Senha com hash Argon2 e política de senha forte
- Todo erro responde em RFC 9457 (Problem Details)
- Log estruturado em JSON com redação de campos sensíveis
- Configuração validada no boot: variável ausente derruba a API com o nome dela

## Rodando a API em Rust

Os valores de desenvolvimento vêm do [Infisical](https://infisical.com), na pasta `/api` do projeto
ligado pelo `.infisical.json` da raiz; nenhum `.env` com valor real fica no disco.

```sh
infisical login
cd apps/api-rs
infisical run --path=/api -- docker compose up -d --wait
infisical run --path=/api -- sqlx migrate run
infisical run --path=/api -- cargo run
```

## Rodando a API em NestJS

Congelada até a migração para Rust terminar. Preencha o `.env` a partir do exemplo; o `compose.yaml`
sobe Postgres e Redis.

```sh
cd apps/api
cp .env.example .env
docker compose up -d
npm ci
npm run migration:run
npm run start
```

Documentação OpenAPI em `http://localhost:3333/api` fora de produção.

## Qualidade

Cada app tem o seu job de CI em todo pull request, com lint, formatação, tipos, build e testes. A API
tem testes de unidade (Jest) e e2e (supertest) contra Postgres e Redis reais, além de um gate que
reprova o build quando as entidades divergem das migrations.

## Documentação

- `docs/requirements.md`: problema, escopo e critérios de aceite
- `docs/decisions/`: ADRs da stack e da arquitetura
- `docs/specs/`: especificações por entrega

# Clinicore

SaaS de gestão para clínicas odontológicas: agenda, prontuário clínico e estético, financeiro, estoque e
laboratório num só lugar, multi-tenant, para redes de clínicas e dentistas autônomos.

> **Em desenvolvimento.** A API já tem a autenticação completa e está sendo migrada de NestJS para Rust,
> sem mudar o contrato HTTP; os módulos de produto vêm em seguida.

## Motivação

O Clinicore nasceu da convivência com a rotina de duas clínicas odontológicas da família: agenda,
prontuário, financeiro e convênio espalhados entre um sistema legado, papel e planilha, e a exportação
de dados que nunca vinha completa. Essas duas clínicas são as pilotos do produto, cada uma operando como
cliente independente, e é contra a rotina real delas que cada módulo é validado antes do lançamento.

## Stack

| App | Papel | Tecnologia |
|---|---|---|
| `apps/api` | contrato HTTP | Rust com axum e sqlx, em migração a partir do NestJS 11 com TypeORM · PostgreSQL 18 · Redis 8 |
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

## Rodando a API

Preencha o `.env` a partir do exemplo; o `compose.yaml` sobe Postgres e Redis.

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

- `docs/requirements/`: o SRS, com problema, escopo e cada requisito com ID e critérios de aceite
- `docs/decisions/`: ADRs da stack e da arquitetura, incluindo a migração da API para Rust (ADR 0009)
- `docs/specs/`: especificações por entrega

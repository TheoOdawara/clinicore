# 0006. API em REST orientado a recurso, com erro em Problem Details

- Status: accepted
- Date: 2026-09-20
- Emenda a: spec `003-autenticacao-api.md` (rotas, regra 10 e catálogo de erros) e
  `003-autenticacao-web.md` (rotas chamadas pelo web)

## Context

A spec 003 desenhou a autenticação como ações sobre HTTP: `POST /auth/sign-in`, `POST /auth/refresh`,
`POST /auth/sign-out`. Cada rota tem um verbo no caminho e quase todas são `POST`. O resto da API
ainda não existe, então nada define o formato das URLs que vêm depois. O `CLAUDE.md` da raiz diz que
cada app escreve a sua regra de URL no próprio `CLAUDE.md`, e o `apps/api/CLAUDE.md` ainda não tem
nenhuma.

O corpo de erro é um formato próprio, `{ code, message, fields }`, nascido na #3. Nenhum cliente
externo o conhece. O `apps/web` ainda não tem código que o leia.

As 5 rotas da #66 existem apenas em commits locais, sem push. É o último momento em que trocar o
contrato não quebra nenhum consumidor.

Ao revisar o contrato, apareceram dois problemas que a forma atual escondia:

- **O `OriginGuard` só checa `POST`** (`common/guards/origin.guard.ts`). Uma rota em `DELETE`, `PUT` ou
  `PATCH` passaria sem proteção de origem.
- **`GET /auth/verify-email?token=` altera estado.** Scanners de link de e-mail, como o Safe Links do
  Outlook, abrem a URL antes da pessoa e consumiriam o token.

## Decision

**A API segue REST orientado a recurso, com a semântica de método e de status da RFC 9110.** A URL é
um substantivo no plural, em kebab-case, e nunca carrega um verbo. O método diz o que acontece:

| Método | Uso |
|---|---|
| `GET` | ler, sem efeito colateral |
| `POST` | criar |
| `PUT` | substituir por inteiro |
| `PATCH` | alterar em parte |
| `DELETE` | remover |

Uma operação sem nome natural de recurso vira um sub-recurso substantivo:

- o refresh gera um novo par de credenciais em `POST /sessions/current/tokens`;
- a confirmação de um token cria uma `confirmation`.

**A única exceção é o OAuth.** `GET /oauth/google` e `GET /oauth/google/callback` são redirecionamentos
de navegador cujo formato vem do protocolo.

As rotas da spec 003 passam a ser:

| Antes | Depois | Status |
|---|---|---|
| `POST /auth/sign-up` | `POST /users` | `202` |
| `POST /auth/sign-in` | `POST /sessions` | `201`, com `Location: /sessions/current` |
| `GET /auth/session` | `GET /sessions/current` | `200` |
| `POST /auth/refresh` | `POST /sessions/current/tokens` | `204` |
| `POST /auth/sign-out` | `DELETE /sessions/current` | `204` |
| `POST /auth/send-verification-email` | `POST /email-verifications` | `202` |
| `GET /auth/verify-email?token=` | `POST /email-verifications/confirmation`, com `{ token }` no corpo | `204` |
| `POST /auth/request-password-reset` | `POST /password-resets` | `202` |
| `POST /auth/reset-password` | `POST /password-resets/confirmation` | `204` |
| `POST /auth/change-password` | `PUT /users/me/password` | `204` |
| `GET /auth/google` | `GET /oauth/google` | `302` |
| `GET /auth/google/callback` | `GET /oauth/google/callback` | `302` |

- **O link de verificação de e-mail aponta para o web**, `${APP_ORIGIN}/verify-email?token=`, do mesmo
  jeito que o link de reset. Quem faz o `POST` é o web.
- **Um token nunca vai no path.** O logger descarta a query, mas registra o path.
- **O cookie `clinicore_refresh` passa a ter `Path=/sessions/current/tokens`.**
- **O `OriginGuard` checa todo método que não é `GET`, `HEAD` nem `OPTIONS`.**
- **O `enableCors` aceita `GET`, `POST`, `PUT`, `PATCH`, `DELETE` e `OPTIONS`.**

**Todo erro sai como Problem Details da RFC 9457**, com `Content-Type: application/problem+json`:

```json
{ "type": "tag:clinicore.com.br,2026:invalid-credentials", "title": "Invalid email or password", "status": 401 }
```

```json
{
  "type": "tag:clinicore.com.br,2026:validation-failed",
  "title": "Validation failed",
  "status": 400,
  "errors": [{ "pointer": "#/password", "code": "weak_password" }]
}
```

```json
{ "type": "about:blank", "title": "Not Found", "status": 404 }
```

Regras de cada membro:

- **`type`**
  - É o identificador primário do erro, e o cliente decide por ele. A RFC 9457 exige isso.
  - Num erro do catálogo, é uma tag URI (RFC 4151) montada a partir do código em kebab-case.
  - Num erro que o framework levanta, ou num 5xx sem código do catálogo, é `about:blank`.
- **`title`**
  - É fixo para cada `type`.
  - Num erro do catálogo, é a mensagem do catálogo.
  - Com `about:blank`, é a frase padrão do status HTTP, como a RFC pede. A mensagem gerada pelo Nest
    não chega ao corpo.
- **`status`** repete o status HTTP da resposta.
- **`errors`** aparece só em `validation-failed`. Traz um item por campo, com `pointer` em JSON Pointer
  (`#/address/zip`) e `code` com a primeira restrição violada. É o formato do exemplo da própria RFC.
- **`detail` e `instance` ficam de fora.** Nenhum erro tem hoje texto próprio de ocorrência, nem
  endereço de ocorrência.
- **O `code` no nível raiz some**, porque repetiria o `type`.

Os códigos em `?error=` dos redirecionamentos do OAuth não mudam, porque essas respostas não têm corpo.

## Consequences

- **Um padrão só para a API inteira.** Toda feature nova modela recurso, e o `apps/api/CLAUDE.md`
  ganha a regra de URL.
- **As 5 rotas da #66 são reescritas antes do push.** O controller, o `Path` do cookie de refresh, os
  testes e o Swagger mudam. Nenhum consumidor quebra, porque nenhum existe ainda.
- **A #67 muda de forma.** A verificação de e-mail passa a ser `POST` a partir do web, e o web ganha a
  rota `/verify-email` que chama a API. Um scanner de link deixa de consumir o token.
- **O `DELETE` e as próximas rotas `PUT` e `PATCH` ficam sob a proteção de origem.** Sem a troca do
  guard, o sign-out em `DELETE` passaria sem ela.
- **O corpo de erro passa a ser lido por qualquer cliente que conheça a RFC 9457.** O web decide pelo
  `type` e escreve o texto a partir dele.
- **A tag URI não leva a uma página.** A RFC permite isso, mas recomenda URI resolvível. Quando existir
  uma página que documente os erros, o `type` passa a ser uma URL `https` que aponta para ela. Isso
  muda o identificador e precisa ser tratado como mudança de contrato.
- **Os testes de contrato de erro e de rota são reescritos** para o corpo e as rotas novas.

## Alternatives considered

- **Manter as ações em `/auth/*` e usar recurso só no domínio** · rejeitada pelo dono do produto, que
  quer um padrão só na API inteira. Deixar a escolha de forma para cada rota vira gosto pessoal.
- **Custom methods no formato do Google (AIP-136, `POST /sessions:refresh`)** · rejeitada: traz de
  volta o verbo na URL, que é justamente o que esta decisão tira.
- **GraphQL** · rejeitada: exige schema e resolver para cada campo, perde o cache HTTP e espalha a
  autorização por campo. É custo sem ganho para um único cliente fazendo CRUD de clínica.
- **gRPC** · rejeitada: o browser não fala gRPC direto e precisaria de um proxy gRPC-Web na frente.
- **tRPC** · rejeitada: depende de o web importar os tipos da API, e o repo proíbe código compartilhado
  entre os apps.
- **Manter `{ code, message, fields }`** · rejeitada: é um formato próprio sem vantagem sobre o padrão,
  e ainda não tem consumidor, então a troca não custa migração.
- **`type` como URL `https` desde já** · rejeitada por ora: a RFC recomenda que essa URL leve a uma
  página de documentação, e essa página não existe.
- **`code` no nível raiz ao lado do `type`** · rejeitada: dois identificadores para o mesmo erro, e a
  RFC manda o cliente decidir pelo `type`.

# 003 — Autenticar e manter sessão (API)

> **Status:** fechada
> **Perfil:** API
> **Módulo:** `apps/api`
> **Epic:** #1 — Plataforma
> **Issue:** #3
> **Spec irmã:** `docs/specs/003-autenticacao-web.md` (perfil UI)

## Acceptance Criteria

### Contrato

O handler do Better Auth é montado com `.mount(auth.handler)` e responde sob o `basePath` padrão
`/api/auth`. Nenhuma dessas rotas é escrita à mão; método e caminho são os do Better Auth.

| Método | Rota | Auth / Role | Idempotente |
| --- | --- | --- | --- |
| `POST` | `/api/auth/sign-up/email` | público | Sim |
| `POST` | `/api/auth/sign-in/email` | público | Não — limitado pela regra 16 |
| `POST` | `/api/auth/sign-in/social` | público | Sim |
| `GET` | `/api/auth/callback/google` | `state` e `code` do Google | Sim |
| `POST` | `/api/auth/sign-out` | sessão válida | Sim |
| `GET` | `/api/auth/get-session` | cookie de sessão | Sim |
| `POST` | `/api/auth/send-verification-email` | público | Sim, na janela da regra 15 |
| `GET` | `/api/auth/verify-email` | token de verificação | Sim |
| `POST` | `/api/auth/request-password-reset` | público | Sim, na janela da regra 15 |
| `GET` | `/api/auth/reset-password/:token` | token de reset | Sim |
| `POST` | `/api/auth/reset-password` | token de reset | Sim |
| `POST` | `/api/auth/change-password` | sessão válida | Sim |
| `GET` | `/health` | público | Sim |

**Critério de idempotência:** uma rota é idempotente quando N chamadas iguais deixam o banco, e o
que sai por e-mail, no mesmo estado que uma chamada. A resposta pode mudar — uma segunda chamada que
responde `400` sem gravar nada continua idempotente. As atualizações da tabela `rateLimit` não entram
no critério: são o mecanismo que limita as chamadas.

| Rota | Por que é idempotente, ou como é limitada |
| --- | --- |
| `sign-up/email` | O e-mail é único. Repetir devolve a mesma resposta genérica, sem gravar e sem enviar (regra 8) |
| `sign-in/email` | **Não é.** Cada login com sucesso cria uma `session` — esse é o propósito da rota. O crescimento é limitado a 5 sessões por usuário (regra 16) |
| `sign-in/social` | O `state` do OAuth vive num cookie, não no banco (regra 6). A chamada não grava nada |
| `callback/google` | O `code` do Google é de uso único. Repetir o callback falha na troca do código e não cria sessão |
| `send-verification-email` | Dentro de 60 segundos, a repetição para o mesmo endereço não envia (regra 15) |
| `verify-email` | Depois da primeira, o e-mail já está verificado. Repetir redireciona sem criar sessão |
| `request-password-reset` | Dentro de 60 segundos, a repetição para o mesmo endereço não grava token nem envia (regra 15) |
| `reset-password/:token` | Só redireciona, sem gravar |
| `reset-password` | O token é consumido na primeira. Repetir responde `400 INVALID_TOKEN` e a senha continua a da primeira |
| `change-password` | Repetir falha em `currentPassword`, porque a senha já mudou |

### Request

**`POST /api/auth/sign-up/email`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `name` | `string` | Sim | 1 a 100 caracteres, após remover espaços das pontas |
| `email` | `string` | Sim | endereço de e-mail válido; gravado em minúsculas |
| `password` | `string` | Sim | regra 4 (política de senha) |
| `callbackURL` | `string` | Não | regra 1; o web sempre envia `${WEB_ORIGIN}/verify-email` |

```json
{ "name": "Ana Souza", "email": "ana@exemplo.com", "password": "Clinica#2026", "callbackURL": "http://localhost:3000/verify-email" }
```

**`POST /api/auth/sign-in/email`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `email` | `string` | Sim | endereço de e-mail válido |
| `password` | `string` | Sim | não vazia |
| `callbackURL` | `string` | Não | regra 1; é o destino do link reenviado pela regra 5, e o web sempre envia `${WEB_ORIGIN}/verify-email` |

**`POST /api/auth/sign-in/social`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `provider` | `string` | Sim | exatamente `google` |
| `callbackURL` | `string` | Não | regra 1; o web sempre envia `${WEB_ORIGIN}/app` |
| `errorCallbackURL` | `string` | Não | regra 1; o web sempre envia `${WEB_ORIGIN}/login` |

**`POST /api/auth/send-verification-email`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `email` | `string` | Sim | endereço de e-mail válido |
| `callbackURL` | `string` | Não | regra 1; o web sempre envia `${WEB_ORIGIN}/verify-email` |

**`POST /api/auth/request-password-reset`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `email` | `string` | Sim | endereço de e-mail válido |
| `redirectTo` | `string` | Não | regra 1; o web sempre envia `${WEB_ORIGIN}/reset-password` |

**Os campos de redirecionamento são contrato do web, não da API.** O Better Auth não os exige; o
web os envia sempre, como fixa a spec irmã. Sem `callbackURL`, o link de verificação leva
`callbackURL=/`, e um link inválido ou expirado responde `401` em JSON em vez de redirecionar
(regra 5).

**`POST /api/auth/reset-password`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `newPassword` | `string` | Sim | regra 4 (política de senha) |
| `token` | `string` | Sim | token de reset não expirado e não consumido |

**`POST /api/auth/change-password`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `currentPassword` | `string` | Sim | senha atual da conta |
| `newPassword` | `string` | Sim | regra 4 (política de senha) |
| `revokeOtherSessions` | `boolean` | Não | o web sempre envia `true` (regra 7); ausente, as outras sessões ficam |

### Response

**`200 OK` — `POST /api/auth/sign-in/email`**

```json
{
  "redirect": false,
  "token": "…",
  "user": {
    "id": "…",
    "name": "Ana Souza",
    "email": "ana@exemplo.com",
    "emailVerified": true,
    "image": null,
    "createdAt": "2026-09-16T12:00:00.000Z",
    "updatedAt": "2026-09-16T12:00:00.000Z"
  }
}
```

Junto, o header `Set-Cookie` com o cookie de sessão da regra 2.

**`200 OK` — `POST /api/auth/sign-up/email`**, tanto para e-mail novo quanto para e-mail já
cadastrado (regra 8):

```json
{ "token": null, "user": { "id": "…", "name": "Ana Souza", "email": "ana@exemplo.com", "emailVerified": false, "image": null, "createdAt": "…", "updatedAt": "…" } }
```

**`200 OK` — `POST /api/auth/request-password-reset`**, para qualquer e-mail, cadastrado ou não, e
dentro ou fora da janela da regra 15:

```json
{ "status": true, "message": "If this email exists in our system, check your email for the reset link" }
```

**`200 OK` — `GET /api/auth/get-session`**

```json
{
  "session": { "id": "…", "token": "…", "userId": "…", "expiresAt": "2026-09-17T12:00:00.000Z", "ipAddress": "…", "userAgent": "…" },
  "user": { "id": "…", "name": "Ana Souza", "email": "ana@exemplo.com", "emailVerified": true, "image": null }
}
```

Sem cookie de sessão válido, o corpo é `null` com status `200` — não `401`.

Todo `id` e todo `userId` nas respostas é um UUID em texto, como `"0b8f2c1e-5a4d-4c3b-9e7f-1a2b3c4d5e6f"`
(regra 12).

**`302 Found` — `GET /api/auth/verify-email`** e **`GET /api/auth/reset-password/:token`**: os dois
respondem por redirecionamento para o `callbackURL` do link. Em erro, o redirecionamento leva
`?error=<code>` na query; em `reset-password/:token` com sucesso, leva `?token=<token>`.

| Status | Quando |
| --- | --- |
| `200` | A operação concluiu, a sessão é inexistente em `get-session`, ou a resposta é genérica das regras 8 e 15 |
| `302` | Links de verificação e de reset, com sucesso ou com `?error=` |
| `400` | Corpo inválido, senha fora da política, token de reset inválido ou expirado, senha atual incorreta em `change-password`, sessão não fresca |
| `401` | Credenciais incorretas; link de verificação inválido ou expirado sem `callbackURL` |
| `403` | E-mail ainda não verificado, `Origin` fora de `trustedOrigins`, ou URL de redirecionamento absoluta fora de `trustedOrigins` |
| `422` | Dois cadastros simultâneos com o mesmo e-mail — o segundo perde no índice único |
| `429` | Limite de requisições da rota excedido (regra 11) |
| `500` | Erro desconhecido, sem detalhe no corpo (regra 10); SMTP fora do ar (regra 13) |

### Perfis e privilégios

| Papel | Permissão | Observação |
| --- | --- | --- |
| Visitante sem sessão | — | Cadastra-se, entra, pede recuperação de senha e verifica e-mail |
| Usuário autenticado | sessão válida | Lê a própria sessão, troca a própria senha e sai |
| Orquestrador de container | — | `GET /health`, antes de existir sessão |

**Não existe papel nesta entrega.** O usuário autenticado não pertence a nenhuma clínica e não tem
função. Papéis, permissões e tenancy são as issues #6 e #7.

---

## Regras de Negócio

### 1. O browser fala com a API em origem cruzada, com credenciais

- `@elysiajs/cors` é registrado antes de qualquer rota, com `origin: env.WEB_ORIGIN` — a **origem
  exata**, nunca `*` e nunca um curinga —, `credentials: true`, `methods: ["GET", "POST", "OPTIONS"]`
  e `allowedHeaders: ["Content-Type"]`.
- O Better Auth recebe `trustedOrigins: [env.WEB_ORIGIN]`, a mesma origem.
- **Validação:** uma requisição com `Origin` diferente de `WEB_ORIGIN` não recebe
  `Access-Control-Allow-Origin` na resposta, e o Better Auth responde `403 INVALID_ORIGIN`.
- **Sem redirecionamento aberto.** Em todo `POST`, o Better Auth confere `callbackURL`, `redirectTo` e
  `errorCallbackURL` contra `trustedOrigins`: caminho relativo passa, URL absoluta de outra origem
  responde `403` com `INVALID_CALLBACK_URL`, `INVALID_REDIRECT_URL` ou `INVALID_ERROR_CALLBACK_URL`.
  O mesmo vale para o `callbackURL` da query em `GET /reset-password/:token`.

### 2. O cookie de sessão

- `httpOnly: true` sempre. O JavaScript do web nunca lê o cookie; quem carrega a sessão é
  `GET /api/auth/get-session` com `credentials: "include"`.
- Em `NODE_ENV === "production"` (homolog e produção): `sameSite: "none"` e `secure: true`, porque o
  web e a API ficam em subdomínios distintos e o cookie viaja entre sites.
- Em `NODE_ENV === "development"`: `sameSite: "lax"` e `secure: false`, porque `http://localhost` não
  aceita `Secure`.
- `advanced.defaultCookieAttributes` carrega esses valores; `advanced.useSecureCookies` acompanha
  `NODE_ENV === "production"`.
- **Sem `crossSubDomainCookies`.** O cookie pertence ao host da API e não é compartilhado com nenhum
  outro subdomínio.

### 3. A sessão dura 24 horas

- `session.expiresIn: 60 * 60 * 24` — 86400 segundos.
- `session.updateAge: 60 * 60` — a expiração é empurrada para frente no máximo uma vez por hora de uso.
- `session.freshAge: 60 * 60`.
- **Não existe "lembrar de mim".** Toda sessão dura o mesmo.
- `session.cookieCache` fica desabilitado.

### 4. A política de senha

Uma senha é aceita quando cumpre **todas** as condições:

- no mínimo 8 e no máximo 128 caracteres;
- ao menos uma letra maiúscula;
- ao menos um dígito;
- ao menos um caractere que não seja letra nem dígito.

- `emailAndPassword.minPasswordLength: 8` e `maxPasswordLength: 128` cobrem o tamanho. **A
  complexidade não tem opção nativa no Better Auth** e é escrita aqui.
- A regra vive em `features/auth/password-policy.ts` como **função pura, testada**, exportando
  `isStrongPassword(password: string): boolean`, que confere as quatro condições, tamanho incluído.
- A aplicação é um `hooks.before`, com `createAuthMiddleware` de `better-auth/api`, que dispara nos
  caminhos `/sign-up/email`, `/reset-password` e `/change-password`. Falhando, lança
  `APIError("BAD_REQUEST", { code: "WEAK_PASSWORD", message: "Password must have at least 8 characters, one uppercase letter, one digit and one special character" })`.
- **A guarda fica no hook, não em cada rota.** As três rotas passam pelo mesmo ponto.
- **O hook roda antes da checagem de tamanho do Better Auth**, que fica dentro do handler. Por isso
  toda senha recusada nessas três rotas sai como `WEAK_PASSWORD`; `PASSWORD_TOO_SHORT` e
  `PASSWORD_TOO_LONG` não são alcançáveis nesta API.

### 5. O e-mail é verificado antes do primeiro login

- `emailAndPassword.requireEmailVerification: true` e `emailAndPassword.autoSignIn: false` — o cadastro
  não cria sessão.
- `emailVerification.sendOnSignUp: true`, `sendOnSignIn: true`, `autoSignInAfterVerification: true`,
  `expiresIn: 3600` (1 hora).
- **O reenvio no login não é automático no Better Auth**: depende de `sendOnSignIn: true`, declarado
  aqui. Com ele, entrar com a senha correta e o e-mail não verificado responde
  `403 EMAIL_NOT_VERIFIED` e reenvia o link, sujeito à regra 15. Senha errada responde `401` antes de
  chegar a esse ponto, sem enviar nada.
- O link de verificação é um JWT assinado com `BETTER_AUTH_SECRET`, não uma linha no banco. Ele leva
  `callbackURL` igual a `${WEB_ORIGIN}/verify-email`:
  - válido → marca `emailVerified = true`, cria a sessão e redireciona para o `callbackURL`;
  - e-mail já verificado → redireciona para o `callbackURL` sem criar sessão;
  - expirado → redireciona com `?error=TOKEN_EXPIRED`;
  - inválido → redireciona com `?error=INVALID_TOKEN`.
- Os dois redirecionamentos de erro dependem de o link levar `callbackURL`. Sem ele, o Better Auth
  responde `401` em JSON com o mesmo código.

### 6. Google e senha são a mesma conta

- `socialProviders.google` com `clientId: env.GOOGLE_CLIENT_ID`,
  `clientSecret: env.GOOGLE_CLIENT_SECRET` e `prompt: "select_account"`.
- A URL de callback registrada no Google Cloud Console é `${BETTER_AUTH_URL}/api/auth/callback/google`.
- `account.accountLinking.enabled: true`, `trustedProviders: ["google"]`,
  `allowDifferentEmails: false`, `allowUnlinkingAll: false`.
- Entrar com Google num e-mail que já tem cadastro por senha **vincula** o provedor à conta existente:
  nasce uma linha em `account` com `providerId = "google"` apontando para o mesmo `userId`, e nenhum
  `user` novo é criado.
- Uma conta criada por Google nasce com `emailVerified = true` e nunca passa pela regra 5. A criação de
  `user` e `account` pelo Google é atômica (regra 12).
- **`account.storeStateStrategy: "cookie"`.** Com o adapter de banco, o padrão do Better Auth é
  `"database"`, que grava uma linha em `verification` a cada `POST /sign-in/social` — uma chamada
  pública, sem sessão, que faria o banco crescer a cada clique. Com `"cookie"`, o `state` viaja num
  cookie cifrado e a rota não grava nada.
- **`allowDifferentEmails: false` impede o sequestro de conta**: um provedor que devolvesse outro
  e-mail não vincula a conta errada.

### 7. Recuperar e trocar senha

- `POST /api/auth/request-password-reset` grava um token em `verification` com
  `identifier = "reset-password:<token>"` e envia o link por `emailAndPassword.sendResetPassword`,
  sujeito à regra 15.
- O link aponta para `GET /api/auth/reset-password/:token?callbackURL=${WEB_ORIGIN}/reset-password`,
  que redireciona para o web com `?token=<token>`, ou com `?error=INVALID_TOKEN` quando o token não
  existe ou expirou.
- `emailAndPassword.resetPasswordTokenExpiresIn: 3600` (1 hora).
- `emailAndPassword.revokeSessionsOnPasswordReset: true` — redefinir a senha derruba todas as sessões.
- O token é **consumido antes** da troca da senha. Duas chamadas simultâneas com o mesmo token: a
  primeira troca, a segunda responde `400 INVALID_TOKEN`.
- **Redefinir a senha de um usuário sem `account` de senha cria essa `account`.** É assim que quem
  entrou só com Google passa a ter também login por senha.
- `POST /api/auth/change-password` é sempre chamado com `revokeOtherSessions: true`: quem troca a senha
  continua logado no dispositivo atual e é deslogado em todos os outros.

### 8. Nenhuma resposta revela quem tem conta

- **Cadastro com e-mail já cadastrado responde `200`**, com `token: null` e um usuário sintético, sem
  gravar nada e sem enviar e-mail. É o comportamento do Better Auth quando
  `requireEmailVerification` está ligado, e a spec o mantém. `emailAndPassword.onExistingUserSignUp`
  **não é configurado**: avisar o dono do e-mail seria um vetor de envio sem limite por endereço.
- `POST /api/auth/request-password-reset` responde `200` com o mesmo corpo para e-mail cadastrado e não
  cadastrado.
- `POST /api/auth/sign-in/email` responde `401 INVALID_EMAIL_OR_PASSWORD` tanto para e-mail inexistente
  quanto para senha errada.
- `POST /api/auth/send-verification-email` sem sessão responde `200` para e-mail inexistente, já
  verificado ou pendente, com tempo mínimo de resposta de 500 ms aplicado pelo próprio Better Auth.
  Com sessão, responde na hora e não há o que esconder: o e-mail é o do próprio usuário.
- **Trade-off aceito:** dois cadastros simultâneos com o mesmo e-mail novo fazem o segundo responder
  `422 FAILED_TO_CREATE_USER`, o que revela que o e-mail acabou de ser cadastrado. A janela é a
  duração de uma transação, e fechá-la exigiria reescrever a rota do Better Auth.

### 9. Todo request é logado, e nenhum segredo sai no log

- `core/logger/` cria a instância do Pino e o plugin de requisição, que se pluga em `onAfterResponse`
  e `onError`. Não existe plugin oficial do Elysia para Pino, e nenhum de terceiro entra.
- O nível vem de `env.LOG_LEVEL`.
- Cada requisição registra método, caminho, status e duração em milissegundos.
- **`GET /health` não é logado.**
- O `redact` do Pino cobre, no mínimo: `req.headers.cookie`, `req.headers.authorization`,
  `res.headers["set-cookie"]`, `req.body.password`, `req.body.newPassword`,
  `req.body.currentPassword` e `req.body.token`.
- **O objeto `env` nunca é passado ao logger.** A connection string e os segredos do ambiente não
  entram em nenhum objeto logado, então não há caminho de `redact` para eles.
- O Pino escreve em `stdout`. O erro de ambiente do boot continua indo cru para `stderr`.
- O worker da regra 17 usa a mesma instância de logger.

### 10. Erro desconhecido vira 500 sem detalhe

- O `onError` global de `core/` é o único lugar do app que conhece HTTP.
- Um erro que ele não reconheça é registrado no Pino com a stack completa e respondido como `500` com
  corpo `{ "code": "INTERNAL_ERROR", "message": "Internal server error" }`.
- **Os erros do Better Auth não chegam aqui.** `.mount(auth.handler)` registra uma rota comum do
  Elysia, então `onAfterResponse` e `onError` rodam para ela e o log da regra 9 cobre `/api/auth`.
  Mas o Better Auth captura os próprios erros e já devolve uma `Response` no formato dele,
  `{ code, message }`; um erro que não é `APIError` sai como `500` com corpo vazio. O `code` é a chave
  comum entre os dois formatos, e é sobre ele que o web traduz.

### 11. Toda rota de autenticação tem limite por IP, e o IP não pode ser forjado

- `rateLimit.enabled: true` nos dois ambientes. O padrão do Better Auth desliga em desenvolvimento, e a
  spec liga explicitamente.
- `rateLimit.storage: "database"` e `modelName: "rateLimit"`. Redis está fora da stack por decisão.
- O incremento é uma escrita condicional: o `UPDATE` só acontece enquanto a contagem está abaixo do
  máximo, e quando a condição falha o Better Auth lê de novo e decide. Requisições concorrentes não
  passam todas por uma leitura desatualizada.
- Limite efetivo de cada rota:

  | Caminho | Janela | Máximo | Origem |
  | --- | --- | --- | --- |
  | `/sign-up/email` | 60 s | 3 | `customRules` |
  | `/sign-in/email` | 60 s | 5 | `customRules` |
  | `/sign-in/social` | 10 s | 3 | regra padrão do Better Auth para `/sign-in*` |
  | `/change-password` | 10 s | 3 | regra padrão do Better Auth para `/change-password*` |
  | `/request-password-reset` | 60 s | 5 | `customRules` |
  | `/reset-password` | 60 s | 5 | `customRules` |
  | `/send-verification-email` | 60 s | 3 | regra padrão do Better Auth |
  | qualquer outro caminho sob `/api/auth` | 10 s | 100 | limite global |

- `customRules` compara o caminho exato: `/reset-password` não alcança `/reset-password/:token`, que
  fica no limite global.
- Excedido, a resposta é `429`.
- **O IP vem da cadeia de proxies confiáveis.** O Better Auth lê o IP só do header
  `x-forwarded-for`, nunca do socket. `advanced.ipAddress.trustedProxies` recebe
  `env.TRUSTED_PROXIES`, e o Better Auth percorre a cadeia da direita para a esquerda, pula os hops
  confiáveis e usa o primeiro que não é. O que o cliente escreve à esquerda do hop que o proxy
  acrescentou nunca é lido.
- Uma cadeia malformada, ou só com hops confiáveis, não resolve IP. Sem IP, o Better Auth **não
  desliga o limite**: agrupa essas requisições num contador compartilhado por caminho.
- **A garantia depende do deploy:** a API só pode ser alcançável através do proxy que acrescenta o hop.
  Exposta direto, um `x-forwarded-for` de valor único forjado seria aceito como IP do cliente. Isso é a
  issue #4 (Fora de Escopo).
- Em `development`, sem proxy, o IP resolve como `127.0.0.1`.

### 12. Persistência e Auditoria

- **O adapter é `prismaAdapter(prisma, { provider: "postgresql", transaction: true })`.** O padrão do
  adapter é `transaction: false`, e com ele nenhum `runWithTransaction` do Better Auth é atômico.
- **Tabelas criadas:** seis, em duas migrations. As tabelas do Better Auth vêm do schema gerado pelo
  CLI (`better-auth generate`) e conferido; `emailDispatch` é desta entrega.

  | Tabela | Migration | Campos |
  | --- | --- | --- |
  | `user` | `add_better_auth` | `id`, `name`, `email` (único), `emailVerified` (bool, padrão `false`), `image` (nulo), `createdAt`, `updatedAt` |
  | `session` | `add_better_auth` | `id`, `token` (único), `expiresAt` (indexado), `ipAddress` (nulo), `userAgent` (nulo), `userId` → `user.id` `ON DELETE CASCADE` (indexado), `createdAt`, `updatedAt` |
  | `account` | `add_better_auth` | `id`, `accountId`, `providerId`, `userId` → `user.id` `ON DELETE CASCADE` (indexado), `accessToken` (nulo), `refreshToken` (nulo), `idToken` (nulo), `accessTokenExpiresAt` (nulo), `refreshTokenExpiresAt` (nulo), `scope` (nulo), `password` (nulo), `createdAt`, `updatedAt` |
  | `verification` | `add_better_auth` | `id`, `identifier` (indexado), `value`, `expiresAt` (indexado), `createdAt`, `updatedAt` |
  | `rateLimit` | `add_better_auth` | `id`, `key` (único), `count`, `lastRequest` (bigint, milissegundos, indexado) |
  | `emailDispatch` | `add_email_dispatch` | `id`, `email` (minúsculas), `kind` (`verification` ou `password_reset`), `createdAt` (indexado); índice composto em `email`, `kind`, `createdAt` |

- Os índices em `session.expiresAt`, `verification.expiresAt` e `rateLimit.lastRequest` **não vêm do
  CLI**: são acrescentados ao schema gerado antes de criar a migration, para que a limpeza da regra 17
  não varra a tabela inteira. Pelo mesmo motivo `emailDispatch.createdAt` tem índice próprio, além do
  composto que atende a regra 15.
- **Toda chave primária é UUID.** O padrão do Better Auth é `id` texto gerado na aplicação; aqui
  `advanced.database.generateId: "uuid"`. Com essa opção, o CLI gera `id String @id @default(dbgenerated("pg_catalog.gen_random_uuid()")) @db.Uuid`
  em todas as tabelas e marca `session.userId` e `account.userId` como `@db.Uuid`; com Postgres, o
  adapter do Prisma deixa o banco gerar o valor. `emailDispatch.id` segue o mesmo formato. Os dois
  comportamentos foram verificados no código do Better Auth.
- A senha vive em `account.password`, com o hash do Better Auth, e nunca em `user`.
- **Cadastro por senha e por Google são atômicos.** Os dois rodam em `runWithTransaction`; com
  `transaction: true`, `user` e `account` são gravados juntos ou nenhum é. O e-mail de verificação do
  cadastro é enviado dentro dessa transação: SMTP fora do ar desfaz o cadastro e a rota responde `500`
  (regra 13).
- **Auditoria:** `N/A` nesta entrega. Não há acesso a prontuário para registrar, e a trilha de
  auditoria é a issue #9.
- **Eventos/integrações disparados:** envio de e-mail por SMTP (regra 13) e o job agendado de limpeza
  (regra 17).

### 13. O e-mail sai direto, sem fila

- `core/mail/` cria o transport do Nodemailer sobre o SMTP da Resend, em todos os ambientes, a partir
  de `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD` e `MAIL_FROM`.
- Dois e-mails, os dois em pt-BR:

  | Gatilho | Assunto | Corpo |
  | --- | --- | --- |
  | Cadastro, login sem verificação e reenvio | "Confirme seu e-mail no Clinicore" | "Olá, {nome}. Confirme seu e-mail para começar a usar o Clinicore. O link expira em 1 hora." + botão "Confirmar e-mail" |
  | Recuperação de senha | "Redefinir sua senha do Clinicore" | "Olá, {nome}. Recebemos um pedido para redefinir sua senha. O link expira em 1 hora. Se não foi você, ignore este e-mail." + botão "Redefinir senha" |

- **O envio é aguardado.** SMTP fora do ar faz a chamada responder `500` com corpo vazio, e isso é
  preferível a uma resposta de sucesso cujo link nunca chega. No cadastro, a transação da regra 12 é
  desfeita junto.
- A fila da regra 17 não é usada para e-mail.

### 14. Toda variável nova é obrigatória e validada no boot

Segue a regra 4 da spec `002`, sem exceção: schema TypeBox em `core/config/env-schema.ts`, `parseEnv`
puro e testado, `env.ts` escrevendo em `stderr` e saindo com `1`, nenhum valor padrão em nenhum
ambiente, o valor recebido nunca impresso. **A API e o worker leem o mesmo módulo** e exigem o mesmo
conjunto de variáveis.

| Nome | Tipo | Formato esperado |
| --- | --- | --- |
| `NODE_ENV` | string | `expected one of: development, production` |
| `BETTER_AUTH_SECRET` | string | `expected a string with at least 32 characters` |
| `BETTER_AUTH_URL` | string | `expected an absolute URL with no trailing slash (https://…)` |
| `GOOGLE_CLIENT_ID` | string | `expected a non-empty string` |
| `GOOGLE_CLIENT_SECRET` | string | `expected a non-empty string` |
| `LOG_LEVEL` | string | `expected one of: fatal, error, warn, info, debug, trace, silent` |
| `SMTP_HOST` | string | `expected a hostname` |
| `SMTP_PORT` | inteiro | `expected an integer between 1 and 65535` |
| `SMTP_USER` | string | `expected a non-empty string` |
| `SMTP_PASSWORD` | string | `expected a non-empty string` |
| `MAIL_FROM` | string | `expected an email address` |
| `TRUSTED_PROXIES` | lista de string | `expected a comma-separated list of CIDR blocks (10.0.0.0/8,…)` |

- `apps/api/src/__tests__/boot.test.ts` afirma hoje que o `stderr` tem exatamente duas linhas quando
  falta `DATABASE_URL`. Com as variáveis acima ele passa a listar todas as ausentes, e o teste é
  atualizado junto — o caso feliz também precisa das novas variáveis no `startServer`.
- `.github/workflows/ci.yml` recebe as mesmas variáveis no bloco `env` do job `api`, e um step
  `prisma migrate deploy` antes dos testes.
- O job `web` recebe `VITE_API_URL`: hoje o `vite build` passa porque `src/shared/env/env.ts` não é
  importado por ninguém, e isso deixa de valer quando o cliente do Better Auth o importar.

### 15. Pedidos de e-mail são limitados por endereço, e não só por IP

O limite por IP da regra 11 não protege uma vítima: de vários IPs, o mesmo endereço receberia e-mails
sem fim, cada pedido de reset gravaria uma linha em `verification`, e a cota de 100 e-mails por dia da
Resend acabaria para todos os usuários.

- **Por endereço, em qualquer IP:** no máximo **1 pedido a cada 60 segundos** e **5 pedidos a cada 24
  horas**, contados separadamente para `verification` e `password_reset`.
- A contagem vive em `emailDispatch`. Cada pedido aceito grava uma linha; o pedido barrado não grava.
- **Duas guardas sobre a mesma contagem**, em
  `features/auth/repository/email-dispatch.repository.ts`, **separadas pelo caminho do request**. Um
  pedido é contado por exatamente uma delas, nunca pelas duas:
  1. **`hooks.before` em `/request-password-reset` e `/send-verification-email`.** Confere e grava.
     Barrado, o hook devolve a resposta genérica da rota — o mesmo status e o mesmo corpo de um pedido
     aceito — sem deixar o Better Auth prosseguir. Nenhuma linha em `verification`, nenhum e-mail. Um
     `hooks.before` que retorna uma resposta interrompe a rota.
  2. **Dentro de `sendVerificationEmail` e `sendResetPassword`**, que recebem o `request` como segundo
     argumento. Quando o caminho do request é um dos dois da guarda 1, o callback envia sem conferir:
     o pedido já foi contado e aceito. Em qualquer outro caminho — `/sign-up/email` e
     `/sign-in/email` — o callback confere e grava; barrado, não envia, e a rota responde exatamente
     como responderia enviando.
- **O pedido conta exista a conta ou não.** Se só pedidos de contas existentes fossem registrados, uma
  resposta barrada revelaria que o e-mail tem cadastro. Por isso a primeira guarda grava o pedido
  antes de o Better Auth consultar o usuário.
- Isso torna as duas rotas **idempotentes dentro da janela de 60 segundos**: repetir não grava e não
  envia. Fora da janela, um novo e-mail é o comportamento pedido por quem clica em reenviar.
- O web mostra o botão de reenvio desabilitado por 60 segundos, mas a guarda é esta, não a do web.

### 16. Cada usuário tem no máximo 5 sessões ativas

- Toda sessão criada — login por senha, callback do Google, verificação de e-mail — passa por
  `internalAdapter.createSession`, que grava pelo mecanismo de hooks do Better Auth. Isso foi verificado
  no código.
- `databaseHooks.session.create.after` chama
  `features/auth/repository/session.repository.ts` → `keepNewestSessions(userId, 5)`, que apaga, dentro
  de um único `$transaction`, todas as sessões do usuário exceto as 5 de `createdAt` mais recente.
- O sexto login derruba a sessão mais antiga; o dispositivo dela recebe `null` em `get-session` na
  próxima chamada.
- Dois logins simultâneos do mesmo usuário podem apagar as mesmas linhas; apagar uma linha já apagada
  não é erro, e o resultado final continua sendo as 5 mais recentes.

### 17. Um job diário apaga o que venceu

O Better Auth só remove uma `session` ou uma `verification` vencida quando alguém tenta lê-la. Linha
que ninguém lê fica para sempre.

- `core/queue/` conecta o pg-boss 12, versão mínima 12.33 — a que tem `TestClock` e a garantia de um
  job por horário verificadas —, ao Postgres de `DATABASE_URL`. O pg-boss cria e mantém o próprio
  schema `pgboss` ao iniciar; esse schema não passa pelo Prisma.
- `src/worker.ts` é a entrada do processo separado: inicia o pg-boss, cria a fila
  `purge-expired-auth-records`, agenda com `boss.schedule("purge-expired-auth-records", "0 3 * * *", null, { tz: "America/Sao_Paulo" })`
  e registra o job com `boss.work`.
- `features/auth/job/purge-expired-auth-records.job.ts` chama o service, como o controller faria; o
  service chama os repositories.
- O job apaga, com `deleteMany` do Prisma:

  | Tabela | Condição |
  | --- | --- |
  | `session` | `expiresAt` anterior a agora |
  | `verification` | `expiresAt` anterior a agora |
  | `rateLimit` | `lastRequest` anterior a agora menos 24 horas |
  | `emailDispatch` | `createdAt` anterior a agora menos 24 horas |

- **O job é idempotente**: rodar duas vezes seguidas apaga na segunda zero linhas. O pg-boss garante
  um único job por horário mesmo com mais de uma instância de worker.
- Cada execução registra no Pino a quantidade apagada por tabela.
- Em desenvolvimento o worker roda por `bun run worker`. O container do worker na stack inteira é a
  issue #4.

---

## Erros

Os códigos vêm do `BASE_ERROR_CODES` do Better Auth, exceto `WEAK_PASSWORD` e `INTERNAL_ERROR`, que
são desta entrega. A `message` é inglês, texto de desenvolvedor. `PASSWORD_TOO_SHORT` e
`PASSWORD_TOO_LONG` existem no Better Auth, mas não saem desta API (regra 4).

| Código | HTTP | Quando | Mensagem |
| --- | --- | --- | --- |
| `INVALID_EMAIL_OR_PASSWORD` | `401` | E-mail inexistente, ou senha incorreta no login | "Invalid email or password" |
| `EMAIL_NOT_VERIFIED` | `403` | Login com a senha correta e o e-mail ainda não verificado | "Email not verified" |
| `FAILED_TO_CREATE_USER` | `422` | Segundo de dois cadastros simultâneos com o mesmo e-mail | "Failed to create user" |
| `WEAK_PASSWORD` | `400` | Senha fora da política da regra 4, inclusive por tamanho | "Password must have at least 8 characters, one uppercase letter, one digit and one special character" |
| `INVALID_TOKEN` | `400` | Token de reset inválido, consumido ou expirado em `POST /reset-password` | "Invalid token" |
| `INVALID_TOKEN` | `302` | Link de verificação ou de reset inválido; vai em `?error=` | — |
| `TOKEN_EXPIRED` | `302` | Link de verificação expirado; vai em `?error=` | — |
| `INVALID_TOKEN` | `401` | Link de verificação inválido sem `callbackURL` | "Invalid token" |
| `TOKEN_EXPIRED` | `401` | Link de verificação expirado sem `callbackURL` | "Token expired" |
| `INVALID_PASSWORD` | `400` | `currentPassword` incorreta em `change-password` | "Invalid password" |
| `SESSION_EXPIRED` | `400` | Sessão expirada em operação que exige sessão fresca | "Session expired. Re-authenticate to perform this action." |
| `INVALID_ORIGIN` | `403` | `Origin` fora de `trustedOrigins` | "Invalid origin" |
| `INVALID_CALLBACK_URL` | `403` | `callbackURL` absoluta fora de `trustedOrigins` | "Invalid callbackURL" |
| `INVALID_REDIRECT_URL` | `403` | `redirectTo` absoluta fora de `trustedOrigins` | "Invalid redirectURL" |
| `INVALID_ERROR_CALLBACK_URL` | `403` | `errorCallbackURL` absoluta fora de `trustedOrigins` | "Invalid errorCallbackURL" |
| — | `429` | Limite da rota excedido (regra 11) | corpo do Better Auth, sem código próprio |
| — | `500` | Erro que não é `APIError` numa rota do Better Auth, como SMTP fora do ar | corpo vazio |
| `INTERNAL_ERROR` | `500` | Erro desconhecido em rota própria da API | "Internal server error" |

## Efeitos Colaterais

- **Persistência:** cadastro de e-mail novo grava uma linha em `user`, uma em `account`
  (`providerId = "credential"`) e uma em `emailDispatch`. Cadastro de e-mail existente não grava nada.
  Login grava uma linha em `session` e apaga as excedentes da regra 16. Logout apaga a linha de
  `session`. Verificação de e-mail marca `user.emailVerified = true`. Pedido de reset aceito grava uma
  linha em `emailDispatch` e uma em `verification`. Reset de senha consome a linha de `verification`,
  atualiza ou cria `account.password` e apaga todas as sessões do usuário. Cada requisição limitada
  atualiza `rateLimit`. O job da regra 17 apaga linhas vencidas.
- **Concorrência:**
  - dois cadastros simultâneos com o mesmo e-mail — o índice único de `user.email` reprova o segundo,
    que responde `422 FAILED_TO_CREATE_USER`;
  - dois usos simultâneos do mesmo token de reset — o token é consumido antes da troca, e o segundo
    responde `400 INVALID_TOKEN`;
  - dois pedidos simultâneos de e-mail para o mesmo endereço dentro da janela — a checagem e a gravação
    em `emailDispatch` acontecem num único `$transaction` no repository, com o índice composto; o
    segundo é barrado;
  - dois logins simultâneos do mesmo usuário — regra 16.
- **Transação:**
  - cadastro por senha — `user`, `account` e o envio do e-mail de verificação numa transação
    (regra 12); a linha de `emailDispatch` é gravada fora dela e fica mesmo se o cadastro for desfeito;
  - cadastro por Google — `user` e `account` numa transação (regra 12);
  - `keepNewestSessions` e a checagem-e-gravação de `emailDispatch` — um `$transaction` cada, nos
    repositories desta entrega.

---

## Cenários de Aceite (Gherkin)

Todos os cenários batem na API real com o Postgres real, no padrão da spec `002`: subprocesso do
`server.ts` e `fetch`, como em `apps/api/src/__tests__/boot.test.ts`. Não se faz mock de Prisma nem do
Better Auth. O transport de e-mail é o único substituído, para que o teste conte os envios.

### Cenário 1 — Cadastro cria o usuário e envia o link (caminho feliz)

```gherkin
Dado que o e-mail `ana@exemplo.com` não existe na tabela `user`
Quando é enviado `POST /api/auth/sign-up/email` com nome, e-mail e a senha `Clinica#2026`
Então o sistema responde `200` com `token` nulo
E existe uma linha em `user` com `email = "ana@exemplo.com"` e `emailVerified = false`
E existe uma linha em `account` com `providerId = "credential"` e `password` preenchida
E `user.id` e `account.userId` são o mesmo UUID, gerado pelo banco
E um e-mail com o assunto `Confirme seu e-mail no Clinicore` foi entregue ao transport
E existe exatamente uma linha em `emailDispatch` para esse endereço, com `kind = "verification"`
E nenhum cookie de sessão é devolvido
```

### Cenário 2 — Cadastro repetido não grava, não envia e não revela a conta (exceção, regra 8)

```gherkin
Dado um usuário cadastrado com `email = "ana@exemplo.com"`
Quando é enviado `POST /api/auth/sign-up/email` com o mesmo e-mail, repetido 3 vezes
Então as 3 respostas são `200` com `token` nulo
E o formato do corpo é o mesmo do Cenário 1
E continua existindo exatamente uma linha em `user` e uma em `account` para esse e-mail
E nenhum e-mail foi entregue ao transport
```

### Cenário 3 — Login antes de verificar é bloqueado e reenvia o link (exceção, regra 5)

```gherkin
Dado um usuário cadastrado com `emailVerified = false` e sem pedido de e-mail nos últimos 60 segundos
Quando é enviado `POST /api/auth/sign-in/email` com a senha correta
Então o sistema responde `403` com o código `EMAIL_NOT_VERIFIED`
E nenhuma linha é criada em `session`
E um e-mail de confirmação foi entregue ao transport
E com a senha errada a resposta é `401 INVALID_EMAIL_OR_PASSWORD` e nenhum e-mail é entregue
```

### Cenário 4 — Verificar o e-mail loga o usuário, e repetir não cria outra sessão (caminho feliz, regra 5)

```gherkin
Dado um usuário cadastrado com `emailVerified = false` e um link de verificação válido
Quando é feita a requisição `GET /api/auth/verify-email` com o token e `callbackURL` igual a `http://localhost:3000/verify-email`
Então o sistema responde `302` para `http://localhost:3000/verify-email`
E `user.emailVerified` passa a ser `true`
E existe exatamente uma linha em `session` para esse usuário
E repetir a mesma requisição responde `302` para o mesmo destino
E continua existindo exatamente uma linha em `session`
```

### Cenário 5 — Link de verificação inválido redireciona com o código (exceção, regra 5)

```gherkin
Dado o `callbackURL` igual a `http://localhost:3000/verify-email`
Quando `GET /api/auth/verify-email` é chamado com um token adulterado
Então o sistema responde `302` para `http://localhost:3000/verify-email?error=INVALID_TOKEN`
E quando é chamado com um token expirado
Então responde `302` para `http://localhost:3000/verify-email?error=TOKEN_EXPIRED`
E nenhuma linha em `user` ou em `session` é alterada
```

### Cenário 6 — Login com senha correta abre sessão de 24 horas (caminho feliz, regras 2 e 3)

```gherkin
Dado um usuário com `emailVerified = true`
Quando é enviado `POST /api/auth/sign-in/email` com a senha correta
Então o sistema responde `200`
E o header `Set-Cookie` traz o cookie de sessão com `HttpOnly`
E a linha criada em `session` tem `expiresAt` 24 horas à frente de `createdAt`, com tolerância de 60 segundos
E `GET /api/auth/get-session` com esse cookie devolve o usuário
```

### Cenário 7 — Senha errada não distingue de e-mail inexistente (exceção, regra 8)

```gherkin
Dado um usuário verificado com o e-mail `ana@exemplo.com`
Quando é enviado `POST /api/auth/sign-in/email` com a senha errada
Então o sistema responde `401` com o código `INVALID_EMAIL_OR_PASSWORD`
E a mesma requisição para o e-mail inexistente `ninguem@exemplo.com` responde `401` com o mesmo código
E os dois corpos de resposta são iguais
```

### Cenário 8 — Logout apaga a sessão, e repetir não muda nada (caminho feliz)

```gherkin
Dado um usuário logado com uma linha em `session`
Quando é enviado `POST /api/auth/sign-out` com o cookie de sessão
Então o sistema responde `200`
E a linha em `session` deixa de existir
E `GET /api/auth/get-session` com o mesmo cookie devolve corpo `null`
E repetir `POST /api/auth/sign-out` com o mesmo cookie não altera nenhuma tabela
```

### Cenário 9 — Senha fraca é recusada nas três rotas (exceção, regra 4)

```gherkin
Dado o hook de política de senha ativo
Quando `sem_maiuscula#1`, `SEM_DIGITO#a`, `SemEspecial1` ou `Aa#1` são enviados como senha
Então cada um responde `400` com o código `WEAK_PASSWORD`
E o mesmo vale nas rotas `/sign-up/email`, `/reset-password` e `/change-password`
E nenhuma linha é gravada em `user`, `account` ou `verification`
E `Clinica#2026` é aceita nas três
```

### Cenário 10 — Google vincula à conta existente em vez de duplicar (caminho alternativo, regra 6)

```gherkin
Dado um usuário verificado com `email = "ana@exemplo.com"` e uma linha em `account` com `providerId = "credential"`
Quando o mesmo e-mail conclui o fluxo do provedor Google
Então continua existindo exatamente uma linha em `user` com esse e-mail
E passa a existir uma segunda linha em `account` com `providerId = "google"` e o mesmo `userId`
E entrar com a senha original continua funcionando
```

### Cenário 11 — Iniciar o login com Google não grava no banco (caminho alternativo, regra 6)

```gherkin
Dado a contagem de linhas de todas as tabelas do schema público
Quando é enviado `POST /api/auth/sign-in/social` com `provider` igual a `google`, 3 vezes
Então cada resposta traz a URL de autorização do Google
E cada resposta traz o cookie do `state`
E nenhuma tabela ganhou linha, exceto `rateLimit`
```

### Cenário 12 — Recuperação de senha não revela quem tem conta (exceção, regras 7 e 8)

```gherkin
Dado um usuário verificado com o e-mail `ana@exemplo.com` e sem pedidos de e-mail nas últimas 24 horas
Quando é enviado `POST /api/auth/request-password-reset` para `ana@exemplo.com` e depois para `ninguem@exemplo.com`
Então as duas requisições respondem `200` com corpos iguais
E um e-mail com o assunto `Redefinir sua senha do Clinicore` foi entregue apenas para `ana@exemplo.com`
E existe exatamente uma linha em `verification` com `identifier` iniciado por `reset-password:`
```

### Cenário 13 — Pedidos repetidos para o mesmo endereço não gravam nem enviam (exceção, regra 15)

```gherkin
Dado um usuário verificado com o e-mail `ana@exemplo.com` e sem pedidos de e-mail nas últimas 24 horas
Quando `POST /api/auth/request-password-reset` para `ana@exemplo.com` é enviado 4 vezes em 10 segundos, cada vez com um IP de proxy confiável diferente
Então as 4 respostas são `200` com corpos iguais
E exatamente 1 e-mail foi entregue ao transport
E existe exatamente 1 linha em `verification` com `identifier` iniciado por `reset-password:`
E existe exatamente 1 linha em `emailDispatch` para esse endereço com `kind = "password_reset"`
```

### Cenário 14 — O teto diário por endereço segura o sexto pedido (exceção, regra 15)

```gherkin
Dado o e-mail `ana@exemplo.com` com 5 linhas em `emailDispatch` de `kind = "verification"` nas últimas 24 horas, a mais recente há 2 minutos
Quando é enviado `POST /api/auth/send-verification-email` para esse endereço
Então o sistema responde `200` com o mesmo corpo de um pedido aceito
E nenhum e-mail é entregue ao transport
E continua havendo 5 linhas em `emailDispatch` para esse endereço
E o mesmo pedido de `kind = "password_reset"` para o mesmo endereço continua sendo aceito
```

### Cenário 15 — O limite por endereço vale também para e-mail sem cadastro (exceção, regra 15)

```gherkin
Dado o e-mail `ninguem@exemplo.com`, que não existe em `user`
Quando `POST /api/auth/request-password-reset` para esse endereço é enviado 2 vezes em 10 segundos
Então existe exatamente 1 linha em `emailDispatch` para esse endereço
E as 2 respostas são iguais às de um endereço cadastrado no Cenário 13
```

### Cenário 16 — Redefinir a senha derruba todas as sessões e não aceita o token de novo (caminho feliz, regra 7)

```gherkin
Dado um usuário verificado com duas linhas em `session` e um token de reset válido
Quando é enviado `POST /api/auth/reset-password` com a nova senha `Outra#Senha9`
Então o sistema responde `200`
E nenhuma linha em `session` resta para esse usuário
E entrar com `Outra#Senha9` funciona
E entrar com a senha antiga responde `401`
E repetir a mesma requisição responde `400` com o código `INVALID_TOKEN`
```

### Cenário 17 — Link de reset leva o token ao web, ou o erro (caminho alternativo, regra 7)

```gherkin
Dado um token de reset válido
Quando `GET /api/auth/reset-password/<token>` é chamado com `callbackURL` igual a `http://localhost:3000/reset-password`
Então o sistema responde `302` para `http://localhost:3000/reset-password?token=<token>`
E com um token inexistente responde `302` para `http://localhost:3000/reset-password?error=INVALID_TOKEN`
E nenhuma das duas chamadas altera a tabela `verification`
```

### Cenário 18 — Quem entrou só com Google ganha login por senha pelo reset (caminho alternativo, regra 7)

```gherkin
Dado uma linha em `user` com `email = "ana@exemplo.com"` e `emailVerified = true`
E uma única linha em `account` para ela, com `providerId = "google"`
Quando é pedido o reset de senha e o token recebido é usado com a senha `Clinica#2026`
Então passa a existir uma linha em `account` com `providerId = "credential"` para esse usuário
E a linha com `providerId = "google"` continua existindo
E entrar com `ana@exemplo.com` e `Clinica#2026` responde `200`
```

### Cenário 19 — Trocar a senha mantém a sessão atual e derruba as outras (caminho feliz, regra 7)

```gherkin
Dado um usuário logado em dois dispositivos, com duas linhas em `session`
Quando é enviado `POST /api/auth/change-password` com a senha atual correta, a nova senha e `revokeOtherSessions: true`
Então o sistema responde `200`
E resta exatamente uma linha em `session`, a do cookie usado na requisição
E repetir a mesma requisição responde `400` com o código `INVALID_PASSWORD`
E a senha continua a da primeira chamada
```

### Cenário 20 — O sexto login derruba a sessão mais antiga (exceção, regra 16)

```gherkin
Dado um usuário verificado com 5 linhas em `session`
Quando é enviado `POST /api/auth/sign-in/email` com a senha correta
Então o sistema responde `200`
E o usuário continua com exatamente 5 linhas em `session`
E a linha de `createdAt` mais antiga deixou de existir
E `GET /api/auth/get-session` com o cookie dessa sessão devolve corpo `null`
E 20 logins seguidos, respeitando a regra 11, deixam o usuário com exatamente 5 linhas em `session`
```

### Cenário 21 — CORS libera a origem exata e nenhuma outra (exceção, regra 1)

```gherkin
Dado a API rodando com `WEB_ORIGIN` igual a `http://localhost:3000`
Quando chega um preflight `OPTIONS /api/auth/sign-in/email` com `Origin: http://localhost:3000`
Então a resposta traz `Access-Control-Allow-Origin: http://localhost:3000`
E traz `Access-Control-Allow-Credentials: true`
E nunca traz `Access-Control-Allow-Origin: *`
E o mesmo preflight com `Origin: http://evil.example` não recebe `Access-Control-Allow-Origin`
```

### Cenário 22 — O limite do login barra a sexta tentativa (exceção, regra 11)

```gherkin
Dado a API com `rateLimit` habilitado e a tabela `rateLimit` vazia
Quando seis requisições `POST /api/auth/sign-in/email` com senha errada chegam do mesmo IP confiável dentro de 60 segundos
Então as cinco primeiras respondem `401`
E a sexta responde `429`
```

### Cenário 23 — Forjar `x-forwarded-for` não escapa do limite (exceção, regra 11)

```gherkin
Dado a API com `TRUSTED_PROXIES` igual a `10.0.0.0/8` e a tabela `rateLimit` vazia
Quando seis requisições `POST /api/auth/sign-in/email` com senha errada são enviadas dentro de 60 segundos
E cada uma traz `x-forwarded-for: <forjado>, 203.0.113.7, 10.0.0.1`, com um `<forjado>` diferente a cada chamada
Então as cinco primeiras respondem `401`
E a sexta responde `429`
E a tabela `rateLimit` tem uma única linha para `/sign-in/email`, com a chave de `203.0.113.7`
```

### Cenário 24 — O log registra a requisição sem vazar segredo (caminho feliz, regra 9)

```gherkin
Dado a API com `LOG_LEVEL` igual a `info`
Quando é enviado `POST /api/auth/sign-in/email` com senha correta
Então a saída em `stdout` contém uma linha JSON com o método, o caminho, o status `200` e a duração
E essa linha não contém a senha enviada
E não contém o valor do header `cookie` nem do `set-cookie`
E uma requisição `GET /health` não produz nenhuma linha de log
```

### Cenário 25 — Erro desconhecido vira 500 sem detalhe (exceção, regra 10)

```gherkin
Dado uma rota própria da API que lança um erro não tratado
Quando ela é chamada
Então o sistema responde `500`
E o corpo é exatamente `{"code":"INTERNAL_ERROR","message":"Internal server error"}`
E a stack completa aparece no log do Pino
E a stack não aparece na resposta
```

### Cenário 26 — Ambiente incompleto derruba o boot listando o que falta (exceção, regra 14)

```gherkin
Dado o ambiente sem `BETTER_AUTH_SECRET` e sem `TRUSTED_PROXIES`
Quando a API é inicializada
Então o processo escreve em `stderr` a linha `Invalid environment:`
E escreve a linha `  BETTER_AUTH_SECRET: expected a string with at least 32 characters`
E escreve a linha `  TRUSTED_PROXIES: expected a comma-separated list of CIDR blocks (10.0.0.0/8,…)`
E sai com código `1`
E não abre a porta HTTP
E o worker, inicializado com o mesmo ambiente, falha da mesma forma
```

### Cenário 27 — O cookie é `Secure` e `SameSite=None` em produção (caminho alternativo, regra 2)

```gherkin
Dado a API inicializada com `NODE_ENV` igual a `production`
Quando um login bem-sucedido devolve o cookie de sessão
Então o `Set-Cookie` traz `Secure`, `HttpOnly` e `SameSite=None`
E com `NODE_ENV` igual a `development` o mesmo login devolve `HttpOnly` e `SameSite=Lax`, sem `Secure`
```

### Cenário 28 — O job apaga o que venceu e nada mais, e repetir não apaga de novo (caminho feliz, regra 17)

```gherkin
Dado uma `session` vencida e uma válida
E uma `verification` vencida e uma válida
E uma linha de `rateLimit` com `lastRequest` de 25 horas atrás e uma de 1 hora atrás
E uma linha de `emailDispatch` de 25 horas atrás e uma de 1 hora atrás
Quando o job `purge-expired-auth-records` é executado
Então só as quatro linhas vencidas deixam de existir
E o log registra 1 linha apagada por tabela
E executar o job de novo apaga 0 linhas
```

### Cenário 29 — O worker agenda o job uma única vez por dia (caminho feliz, regra 17)

```gherkin
Dado o worker iniciado com o `TestClock` do pg-boss
Quando o relógio avança até 03:00 de `America/Sao_Paulo`
Então exatamente um job `purge-expired-auth-records` é criado
E iniciar uma segunda instância do worker não cria um segundo job para o mesmo horário
```

### Cenário 30 — SMTP fora do ar desfaz o cadastro (exceção, regras 12 e 13)

```gherkin
Dado que o e-mail `ana@exemplo.com` não existe na tabela `user`
E o transport de e-mail falha ao enviar
Quando é enviado `POST /api/auth/sign-up/email` com nome, e-mail e a senha `Clinica#2026`
Então o sistema responde `500` com corpo vazio
E não existe linha em `user` nem em `account` para esse e-mail
```

### Cenário 31 — URL de redirecionamento de outra origem é recusada (exceção, regra 1)

```gherkin
Dado um usuário verificado com o e-mail `ana@exemplo.com`
Quando é enviado `POST /api/auth/request-password-reset` com `redirectTo` igual a `http://evil.example/reset-password`
Então o sistema responde `403` com o código `INVALID_REDIRECT_URL`
E nenhuma linha é criada em `verification`
E nenhum e-mail é entregue ao transport
E `POST /api/auth/sign-up/email` com `callbackURL` igual a `http://evil.example/verify-email` responde `403` com o código `INVALID_CALLBACK_URL`
E `redirectTo` igual ao caminho relativo `/reset-password` é aceito
```

---

## Fora de Escopo

- **Organização, clínica, rede, papéis e permissões** — issues #6 e #7.
- **`BusinessError` e o formato `{ code, message, fields }`** — issue #6, com o primeiro service que
  tenha regra de negócio própria respondendo por uma rota. O `onError` global nasce aqui, porque o
  logger precisa do hook e o erro desconhecido precisa virar `500`.
- **Convite de usuário** — decidido que não existe: o cadastro é público. A issue #8 precisa ser
  reescrita como gestão de usuários já cadastrados.
- **E-mail pela fila** — a fila desta entrega serve só ao job de limpeza (regra 13).
- **Trilha de auditoria de acesso ao prontuário** — issue #9.
- **2FA, sessões ativas por dispositivo, revogar sessão específica, excluir conta, trocar e-mail e
  editar perfil** — não pedidos.
- **Outros provedores sociais além do Google** — não pedidos.
- **Limite de requisições em `/health`** — decidido deixar de fora.
- **Dockerfile, `compose.yaml` da raiz, container do worker e deploy de homolog** — issue #4, que
  também define o valor de `TRUSTED_PROXIES` em homolog e garante que a API só é alcançável pelo proxy
  que acrescenta o hop em `x-forwarded-for` (regra 11).

## Quebra em Tasks

| # | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- |
| 1 | Add the authentication environment, the logger and the global error handler to apps/api | `core/config/env-schema.ts` e `env.ts` com as 12 variáveis, `core/logger/` com o plugin de Pino, `onError` global em `core/`, `src/__tests__/boot.test.ts` atualizado, `.github/workflows/ci.yml` com as variáveis novas | Cenários 24, 25 e 26 verdes, sem a linha do worker no 26; os quatro gates da API saem com código 0 | — |
| 2 | Authenticate with email and password through Better Auth | `core/auth/` com `prismaAdapter` em `transaction: true`, `features/auth/password-policy.ts` e o `hooks.before` de senha, CORS em `server.ts`, migration `add_better_auth`, `features/auth/repository/session.repository.ts` e o `databaseHooks` de sessão, step `prisma migrate deploy` no CI | Cenários 6, 7, 8, 9, 20, 21 e 27 verdes | 1 |
| 3 | Send the verification and the reset emails with a per-address limit | `core/mail/` e os dois templates, `requireEmailVerification`, `sendOnSignIn`, `sendResetPassword`, migration `add_email_dispatch`, `features/auth/repository/email-dispatch.repository.ts` e as duas guardas da regra 15, separadas pelo caminho do request | Cenários 1, 2, 3, 4, 5, 12, 13, 14, 15, 16, 17, 18, 30 e 31 verdes | 2 |
| 4 | Sign in with Google and link it to the existing account | `socialProviders.google`, `account.accountLinking` e `storeStateStrategy: "cookie"` em `core/auth/` | Cenários 10 e 11 verdes | 2 |
| 5 | Rate-limit the authentication routes by trusted client IP | `rateLimit` com `customRules` e `advanced.ipAddress.trustedProxies` em `core/auth/` | Cenários 22 e 23 verdes | 2 |
| 6 | Change the password of the signed-in user | `change-password` com `revokeOtherSessions`, coberto pelo hook de política da task 2 | Cenário 19 verde | 2 |
| 7 | Purge expired authentication records daily in a worker | `core/queue/` com pg-boss, `src/worker.ts`, script `worker`, `features/auth/job/purge-expired-auth-records.job.ts`, service e repositories da limpeza | Cenários 28 e 29 verdes; linha do worker no Cenário 26 verde | 3, 5 |

As tasks 8, 9 e 10, do `apps/web`, estão na spec irmã `docs/specs/003-autenticacao-web.md`.

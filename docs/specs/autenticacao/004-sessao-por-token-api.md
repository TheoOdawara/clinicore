# 004 — Emitir a sessão por token para o app e entrar com Google nativo e com passkey (API)

> **Status:** publicada
> **Perfil:** API
> **Módulo:** `apps/api`, feature `auth`
> **Epic:** #91, dentro da #1 — Plataforma; as tasks de passkey em #92, dentro da #90 — Beta com a clínica piloto
> **Spec irmã:** `docs/specs/autenticacao/004-autenticacao-mobile.md` (perfil UI)
> **Base:** `docs/specs/autenticacao/003-autenticacao-api.md`. Tudo que esta spec não altera continua
> valendo como está lá; as regras daqui citam as de lá por número.
> **Decisões:** `docs/decisions/0006-api-rest-e-problem-details.md` ·
> `docs/decisions/0007-app-nativo-em-flutter-com-offline.md`

## Acceptance Criteria

### Contrato

| Método | Rota | Auth / Role | Idempotente |
| --- | --- | --- | --- |
| `POST` | `/sessions` | público | Não — regra 16 da 003 |
| `POST` | `/sessions/current/tokens` | cookie `clinicore_refresh`, ou `refreshToken` no corpo com `Clinicore-Client: mobile` | Não — rotaciona (regra 3 da 003) |
| `DELETE` | `/sessions/current` | sessão (regra 2) | Sim |
| `GET` | `/sessions/current` | sessão (regra 2) | Sim |
| `PUT` | `/users/me/password` | sessão (regra 2) | Sim |
| `POST` | `/oauth/google/sessions` | público, só com `Clinicore-Client: mobile` | Não — cria sessão |
| `POST` | `/passkey-registration-challenges` | sessão | Não — substitui o desafio anterior |
| `POST` | `/passkeys` | sessão | Não — cria credencial |
| `GET` | `/passkeys` | sessão | Sim |
| `DELETE` | `/passkeys/{id}` | sessão | Sim |
| `POST` | `/passkey-authentication-challenges` | público | Não — cria desafio |
| `POST` | `/passkey-assertions` | público | Não — cria sessão |

As cinco primeiras rotas já existem (spec 003) e ganham o transporte por token. As sete últimas nascem
aqui. **Sessão** quer dizer: cookie `clinicore_access` válido, ou `Authorization: Bearer <access token>`
com `Clinicore-Client: mobile` (regra 2).

### Request

**Header `Clinicore-Client`**, aceito em toda rota:

| Valor | Efeito |
| --- | --- |
| ausente | transporte por cookie, exatamente como na 003 |
| `mobile` | transporte por token (regra 2) |
| qualquer outro | `400 INVALID_CLIENT` antes do controller |

**`POST /sessions/current/tokens`** com `Clinicore-Client: mobile`

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `refreshToken` | `string` | Sim | `<uuid>.<43 caracteres base64url>` |

Sem o header, a rota não aceita corpo e lê o cookie, como na 003.

**`POST /oauth/google/sessions`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `idToken` | `string` | Sim | 1 a 4096 caracteres |

**`POST /passkeys`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `response` | `object` | Sim | `RegistrationResponseJSON` do WebAuthn: `id`, `rawId`, `type` igual a `public-key`, `response.clientDataJSON` e `response.attestationObject` em base64url |

**`POST /passkey-assertions`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `response` | `object` | Sim | `AuthenticationResponseJSON` do WebAuthn: `id`, `rawId`, `type` igual a `public-key`, `response.clientDataJSON`, `response.authenticatorData` e `response.signature` em base64url |

**`DELETE /passkeys/{id}`** — `id` no path, UUID.

**`POST /passkey-registration-challenges`**, **`POST /passkey-authentication-challenges`** e
**`GET /passkeys`** — sem corpo.

### Response

**`201 Created` — `POST /sessions`, `POST /oauth/google/sessions` e `POST /passkey-assertions`, com
`Clinicore-Client: mobile`.** Nenhum `Set-Cookie`.

```json
{
  "user": {
    "id": "0b8f2c1e-5a4d-4c3b-9e7f-1a2b3c4d5e6f",
    "name": "Ana Souza",
    "email": "ana@exemplo.com",
    "emailVerified": true,
    "image": null
  },
  "tokens": {
    "accessToken": "<JWT>",
    "refreshToken": "<sessionId>.<segredo>",
    "accessTokenExpiresIn": 900
  }
}
```

Sem o header, `POST /sessions` e `POST /passkey-assertions` respondem o corpo da 003 — só `user` — com
os dois `Set-Cookie`.

**`200 OK` — `POST /sessions/current/tokens` com `Clinicore-Client: mobile`**: `{ "tokens": { … } }`,
no mesmo formato acima. Sem o header continua `204` com os cookies.

**`201 Created` — `POST /passkey-registration-challenges`**: o `PublicKeyCredentialCreationOptionsJSON`
do WebAuthn, como sai de `generateRegistrationOptions`. **`201 Created` —
`POST /passkey-authentication-challenges`**: o `PublicKeyCredentialRequestOptionsJSON`, como sai de
`generateAuthenticationOptions`. Os dois são documentados no OpenAPI como objeto livre, porque o formato
é o da especificação WebAuthn e não um DTO nosso.

**`201 Created` — `POST /passkeys`** e item de **`200 OK` — `GET /passkeys`**:

```json
{ "id": "7c1d…", "createdAt": "2026-09-21T13:00:00.000Z", "lastUsedAt": null }
```

`GET /passkeys` responde `{ "passkeys": [ … ] }`, do `createdAt` mais recente para o mais antigo.

**`204 No Content`** — `DELETE /passkeys/{id}`.

| Status | Quando |
| --- | --- |
| `200` | Refresh do app; lista de passkeys |
| `201` | Sessão criada; desafio criado; passkey cadastrada |
| `204` | Passkey removida; as rotas da 003 que já respondiam `204` |
| `400` | `Clinicore-Client` inválido; corpo fora do DTO; registro de passkey recusado |
| `401` | Sessão ausente ou inválida; token do Google inválido; asserção de passkey recusada |
| `403` | E-mail do Google não verificado; `Origin` recusada (regra 4) |
| `404` | Passkey inexistente ou de outro usuário |
| `409` | Limite de 10 passkeys atingido |
| `429` | Limite da rota (regra 8) |

### Perfis e privilégios

| Papel | Permissão | Observação |
| --- | --- | --- |
| Visitante sem sessão | — | Entra por senha, Google nativo ou passkey |
| Usuário autenticado | sessão válida | Lista, cadastra e remove as próprias passkeys |

**Não existe papel nesta entrega**, como na 003. Papéis e permissões são as issues #6 e #7.

---

## Regras de Negócio

### 1. O cliente se declara pelo header `Clinicore-Client`

- **As mesmas rotas atendem web e app.** O que muda é só o transporte da sessão, e quem escolhe é o
  cliente, **declarando** — nunca a API adivinhando por `User-Agent` ou pela falta de `Origin`, que
  qualquer um forja e que esconderia o comportamento.
- `common/decorators/session-client.decorator.ts` expõe `@SessionClient()`, que lê o header e devolve
  `"web"` ou `"mobile"`. Valor diferente de `mobile` responde `400 INVALID_CLIENT`.
- `features/auth/enums/session-client.enum.ts` espelha a coluna da regra 3.
- **O header só muda as rotas do Contrato.** `POST /email-verifications/confirmation` continua criando
  a sessão por cookie sempre: o link do e-mail abre no web (spec irmã, regra 9).

### 2. Com `Clinicore-Client: mobile`, a sessão viaja no corpo e no header `Authorization`

- **Login** — `POST /sessions`, `POST /oauth/google/sessions` e `POST /passkey-assertions` devolvem
  `tokens` no corpo e **nunca** escrevem cookie.
- **Uso** — o app manda `Authorization: Bearer <accessToken>`. O `jwt.strategy.ts` passa a extrair o
  token assim: com `Clinicore-Client: mobile`, **só** do header `Authorization`; sem ele, **só** do
  cookie `clinicore_access`. Um cliente nunca é autenticado pelo transporte do outro.
- **Refresh** — `POST /sessions/current/tokens` com `Clinicore-Client: mobile` lê `refreshToken` do
  corpo e responde `200` com `tokens`. A rotação e a detecção de reúso são as da regra 3 da 003, sem
  mudança: o segredo é o mesmo `<sessionId>.<segredo>`, gravado como SHA-256.
- **Logout** — `DELETE /sessions/current` com Bearer apaga a sessão e grava a denylist, como na regra 2
  da 003, e responde `204` sem `Set-Cookie`.
- O JWT ganha o claim `cli`, com o cliente gravado na sessão (`web` ou `mobile`), ao lado de `sub`, `sid` e
  `exp`: 900 segundos, a mesma denylist no Redis. A strategy recusa com `401 INVALID_SESSION` o token cujo
  `cli` difere do transporte declarado — é o que impede o token do app de autenticar pelo cookie, e o do web
  pelo Bearer. Um access token emitido antes do claim cai uma vez em `401` e o web o renova pelo refresh.
- `accessTokenExpiresIn` é sempre `900`. O app não decodifica o JWT.

### 3. A sessão do app vive 7 dias de inatividade; a do web continua em 24 horas

- `session` ganha a coluna `client`, enum `session_client_enum` com `web` e `mobile`, obrigatória,
  padrão `web` para as linhas que já existem. Migration `AddSessionClient`.
- `createSession(userId, refreshTokenHash, ip, userAgent, client)` recebe o cliente explicitamente, e
  `expiresAt` nasce em **86400 segundos** para `web` e **604800 segundos** para `mobile`. Cada rotação
  empurra `expiresAt` pelo tempo do cliente gravado na sessão, nunca pelo header da requisição de
  refresh.
- **O web fica em 24 horas porque o computador da recepção é compartilhado**; o celular é pessoal.
- O teto de 5 sessões da regra 16 da 003 conta web e app juntos.
- O cookie de refresh do web não muda: `Max-Age` 86400.

### 4. A guarda de `Origin` deixa passar quem não tem cookie

- A `OriginGuard` (regra 1 da 003) passa a aceitar um método que não é `GET`, `HEAD` nem `OPTIONS`
  quando **uma** destas vale:
  1. o header `Origin` pertence a `ALLOWED_ORIGINS`, como hoje;
  2. **não há header `Origin` e não há header `Cookie`.**
- Todo o resto continua `403 INVALID_ORIGIN`: `Origin` fora da lista, com ou sem cookie; `Origin`
  ausente com cookie.
- **Por quê:** CSRF depende de o browser anexar um cookie sozinho. Uma requisição sem cookie não tem
  credencial ambiente para sequestrar — o app autentica por Bearer, que o browser nunca anexa. E o
  browser sempre manda `Origin` num `POST` de outra origem, então a regra 2 não abre porta para página
  nenhuma.

### 5. O app entra com Google pelo `idToken` do SDK nativo

- `POST /oauth/google/sessions` exige `Clinicore-Client: mobile`; sem ele, `400 INVALID_CLIENT`. O web
  continua no fluxo de redirecionamento da regra 6 da 003.
- A API verifica o token com `google-auth-library` 11.1.0:
  `new OAuth2Client().verifyIdToken({ idToken, audience: env.GOOGLE_CLIENT_ID })`. **A audiência é o
  client ID web que a API já tem**: o app passa esse ID como `serverClientId` ao `google_sign_in`, e o
  Google emite o `idToken` endereçado a ele no Android e no iOS. **Nenhuma variável nova.**
- Assinatura, expiração ou audiência inválida responde `401 INVALID_PROVIDER_TOKEN`.
- `email_verified` diferente de `true` responde `403 UNVERIFIED_PROVIDER_EMAIL` e não grava nada.
- Daí em diante é **o mesmo método de serviço do callback da #68**: vincula ao `user` existente pelo
  e-mail, ou cria `user` e `account` com `provider = "google"` e `providerAccountId = sub` numa
  transação, com `emailVerified = true`. Nenhum token do Google é guardado.
- Cria a sessão com `client = mobile` e aplica o teto de 5.

### 6. Passkey: cadastro por quem já está logado

- Biblioteca: `@simplewebauthn/server` 14.0.2.
- **RP ID é o host de `APP_ORIGIN`**, `app.clinicore.com.br` — derivado, sem variável nova. `rpName` é
  a constante `Clinicore`.
- **Origens aceitas:** `APP_ORIGIN` e cada item de `PASSKEY_ANDROID_ORIGINS` (regra 9), que carrega o
  `android:apk-key-hash:<…>` de cada certificado que assina o app.
- `POST /passkey-registration-challenges` chama `generateRegistrationOptions` com `userName` igual a
  `user.email`, `userDisplayName` igual a `user.name`, `userID` com os bytes de `user.id`,
  `attestationType: "none"`, `excludeCredentials` com as passkeys do usuário e
  `authenticatorSelection: { residentKey: "required", userVerification: "required" }`. Grava o
  desafio em `auth:passkey:registration:<userId>` no Redis, com TTL de 300 segundos, **substituindo**
  qualquer desafio anterior do mesmo usuário.
- Com 10 passkeys já cadastradas, a rota responde `409 PASSKEY_LIMIT_REACHED` e não gera desafio.
- `POST /passkeys` lê e apaga o desafio com `GETDEL`, e chama `verifyRegistrationResponse` com
  `expectedChallenge`, as origens aceitas, `expectedRPID` e `requireUserVerification: true`. Desafio
  ausente, vencido ou já usado, ou verificação que falha, responde `400 INVALID_PASSKEY_REGISTRATION`.
- A gravação confere o teto de 10 **na mesma transação** que insere a linha; o 11º responde
  `409 PASSKEY_LIMIT_REACHED`.

### 7. Passkey: login sem digitar o e-mail

- `POST /passkey-authentication-challenges` chama `generateAuthenticationOptions` com o RP ID,
  `userVerification: "required"` e **sem `allowCredentials`** — o aparelho mostra as passkeys que tem
  para o domínio. Grava `auth:passkey:authentication:<challenge>` no Redis com TTL de 300 segundos.
- `POST /passkey-assertions`:
  1. lê o `challenge` do `clientDataJSON` e o consome com `GETDEL`;
  2. busca a passkey por `credentialId` igual ao `response.id`;
  3. chama `verifyAuthenticationResponse` com o desafio, as origens aceitas, o RP ID, a credencial
     gravada e `requireUserVerification: true`;
  4. grava o `counter` novo e `lastUsedAt`, e cria a sessão pelo cliente do header (regra 2), com o
     teto de 5.
- **Desafio ausente ou usado, credencial desconhecida e assinatura inválida respondem igual**:
  `401 INVALID_PASSKEY_ASSERTION`. A resposta nunca diz se a credencial existe.
- Uma passkey só existe para quem já entrou, então o `user` dela sempre tem `emailVerified = true`.
- **Passkey não muda com a senha.** Redefinir ou trocar a senha não apaga passkey, e remover uma
  passkey não derruba sessão.

### 8. Limites por IP

Somam-se à tabela da regra 11 da 003:

| Caminho | Janela | Máximo |
| --- | --- | --- |
| `/oauth/google/sessions` | 60 s | 10 |
| `/passkey-registration-challenges` | 60 s | 10 |
| `/passkeys` (`POST`) | 60 s | 10 |
| `/passkey-authentication-challenges` | 60 s | 30 |
| `/passkey-assertions` | 60 s | 10 |

`/passkey-authentication-challenges` tem 30 porque o web e o app pedem um desafio a cada abertura da
tela Entrar, e uma clínica inteira sai pelo mesmo IP.

### 9. Variável nova

Segue a regra 14 da 003: obrigatória, validada no boot, sem valor padrão.

| Nome | Tipo | Formato esperado |
| --- | --- | --- |
| `PASSKEY_ANDROID_ORIGINS` | lista de string | `expected a comma-separated list of android:apk-key-hash:<base64url> origins` |

- O decorator confere cada item com o prefixo `android:apk-key-hash:` seguido de 43 caracteres
  base64url, sem regex à mão fora dele.
- O job `api` do CI recebe um valor fixo de teste no bloco `env`.

### 10. Log

- O `redact` do Pino (regra 9 da 003) passa a cobrir também `req.body.refreshToken`,
  `req.body.idToken` e `req.body.response`. `req.headers.authorization` já estava coberto.
- O corpo da resposta nunca é registrado, então os tokens do login não chegam ao log.

### 11. Persistência e Auditoria

- **Tabelas/colunas alteradas:**

  | Tabela | Migration | Colunas |
  | --- | --- | --- |
  | `session` | `AddSessionClient` | `client` (`web` ou `mobile`, padrão `web`) |
  | `passkey` | `AddPasskey` | `id`, `userId` → `user.id` `ON DELETE CASCADE` (indexado), `credentialId` (varchar 1024, único), `publicKey` (bytea), `counter` (bigint), `transports` (array de varchar, nulo), `deviceType` (`singleDevice` ou `multiDevice`), `backedUp` (bool), `createdAt`, `lastUsedAt` (nulo) |

  Entities em `features/auth/entities/`, migrations geradas e conferidas pelo gate da regra 12 da 003.
- **Redis:** `auth:passkey:registration:<userId>` e `auth:passkey:authentication:<challenge>`, TTL de
  300 segundos, consumidos por `GETDEL`.
- **Auditoria:** `N/A` — não há acesso a prontuário; a trilha é a issue #9.
- **Eventos/integrações disparados:** verificação do `idToken` contra as chaves públicas do Google, que
  a `google-auth-library` baixa e guarda em cache.

---

## Erros

Somam-se ao catálogo da 003, no mesmo formato de Problem Details.

| Código | HTTP | Quando | Mensagem |
| --- | --- | --- | --- |
| `INVALID_CLIENT` | `400` | `Clinicore-Client` com valor diferente de `mobile`; `POST /oauth/google/sessions` sem o header | "Invalid client" |
| `INVALID_PASSKEY_REGISTRATION` | `400` | Desafio de cadastro ausente, vencido ou usado; verificação do cadastro falhou | "Invalid passkey registration" |
| `INVALID_PROVIDER_TOKEN` | `401` | `idToken` com assinatura, expiração ou audiência inválida | "Invalid provider token" |
| `INVALID_PASSKEY_ASSERTION` | `401` | Desafio ausente ou usado, credencial desconhecida, assinatura inválida | "Invalid passkey assertion" |
| `UNVERIFIED_PROVIDER_EMAIL` | `403` | `idToken` com `email_verified` diferente de `true` | "Provider email not verified" |
| `PASSKEY_NOT_FOUND` | `404` | `DELETE /passkeys/{id}` com passkey inexistente ou de outro usuário | "Passkey not found" |
| `PASSKEY_LIMIT_REACHED` | `409` | Usuário já tem 10 passkeys | "Passkey limit reached" |

`UNVERIFIED_PROVIDER_EMAIL` já existia como `?error=` do redirecionamento; aqui ganha também o corpo.

## Efeitos Colaterais

- **Persistência:** login do app grava `session` com `client = mobile`. Google nativo com e-mail novo
  grava `user` e `account`; com e-mail existente e sem vínculo, só `account`. Cadastro de passkey grava
  `passkey`; login por passkey atualiza `counter` e `lastUsedAt`; remoção apaga a linha.
- **Concorrência:**
  - dois `POST /passkeys` simultâneos com 9 passkeys cadastradas — o teto é conferido na transação, e o
    segundo responde `409`;
  - dois `POST /passkey-assertions` com o mesmo desafio — o `GETDEL` entrega o desafio a um só, e o
    outro responde `401`;
  - dois pedidos de desafio de cadastro seguidos — vale o último, e o cadastro feito sobre o primeiro
    responde `400`.
- **Transação:** inserção de passkey com a conferência do teto; atualização do `counter` com a criação
  da sessão e o corte do teto de sessões; o vínculo do Google, como na regra 6 da 003.

---

## Cenários de Aceite (Gherkin)

Mesmo padrão da 003: `Test.createTestingModule` e `supertest` contra o Postgres e o Redis reais. Duas
fronteiras são substituídas: a verificação do `idToken`, por um `OAuth2Client` de teste que assina com
uma chave própria, e o autenticador WebAuthn, por respostas geradas com uma chave de teste no próprio
teste.

### Cenário 1 — Login do app devolve tokens e nenhum cookie (caminho feliz, regras 1 e 2)

```gherkin
Dado um usuário verificado com a senha `Clinica#2026`
Quando é enviado `POST /sessions` com `Clinicore-Client: mobile` e a senha correta
Então o sistema responde `201` com `user` e `tokens`
E `tokens.accessTokenExpiresIn` é `900`
E nenhum `Set-Cookie` é devolvido
E a `session` criada tem `client = "mobile"` e `expiresAt` 604800 segundos à frente
E `GET /sessions/current` com `Authorization: Bearer <accessToken>` e o header responde `200`
```

### Cenário 2 — Sem o header, nada muda para o web (caminho alternativo, regra 1)

```gherkin
Dado o mesmo usuário
Quando é enviado `POST /sessions` sem `Clinicore-Client`
Então o sistema responde `201` só com `user` e os dois `Set-Cookie` da 003
E a `session` criada tem `client = "web"` e `expiresAt` 86400 segundos à frente
```

### Cenário 3 — Valor desconhecido no header é recusado (exceção, regra 1)

```gherkin
Quando é enviado `POST /sessions` com `Clinicore-Client: desktop`
Então o sistema responde `400` com o código `INVALID_CLIENT`
E nenhuma `session` é criada
```

### Cenário 4 — Um transporte não autentica pelo outro (exceção, regra 2)

```gherkin
Dado um access token válido de uma sessão do app
Quando `GET /sessions/current` é enviado com o token no cookie `clinicore_access` e sem o header
Então o sistema responde `401 INVALID_SESSION`
E com `Authorization: Bearer` mas sem `Clinicore-Client: mobile` também responde `401 INVALID_SESSION`
```

### Cenário 5 — O refresh do app rotaciona e empurra 7 dias (caminho feliz, regras 2 e 3)

```gherkin
Dado uma sessão do app
Quando é enviado `POST /sessions/current/tokens` com o header e `{ refreshToken }`
Então o sistema responde `200` com `tokens` novos
E `expiresAt` da sessão fica 604800 segundos à frente
E reenviar o `refreshToken` antigo responde `401 SESSION_REUSED` e apaga a sessão
E o access token da sessão derrubada responde `401 INVALID_SESSION`
```

### Cenário 6 — Logout do app revoga na hora (caminho feliz, regra 2)

```gherkin
Dado uma sessão do app
Quando é enviado `DELETE /sessions/current` com Bearer e o header
Então o sistema responde `204` sem `Set-Cookie`
E a linha de `session` deixa de existir
E o mesmo access token responde `401 INVALID_SESSION` na requisição seguinte
```

### Cenário 7 — A guarda de `Origin` aceita o app e segura o CSRF (exceção, regra 4)

```gherkin
Quando `POST /sessions` chega sem `Origin` e sem `Cookie`
Então passa pela guarda
Quando `DELETE /sessions/current` chega sem `Origin` e com o cookie `clinicore_access`
Então o sistema responde `403 INVALID_ORIGIN`
Quando `POST /sessions` chega com `Origin: https://evil.example` e sem cookie
Então o sistema responde `403 INVALID_ORIGIN`
```

### Cenário 8 — Google nativo cria a conta ou vincula à existente (caminho feliz, regra 5)

```gherkin
Dado um `idToken` válido com audiência `GOOGLE_CLIENT_ID`, `email_verified` verdadeiro e o e-mail de um usuário cadastrado por senha
Quando é enviado `POST /oauth/google/sessions` com o header e `{ idToken }`
Então o sistema responde `201` com `user` e `tokens`
E existe uma `account` com `provider = "google"` para esse `user`, e nenhum `user` novo
E com um e-mail sem cadastro, `user` e `account` nascem com `emailVerified = true`
```

### Cenário 9 — Token do Google inválido ou sem e-mail verificado (exceção, regra 5)

```gherkin
Quando `POST /oauth/google/sessions` recebe um `idToken` com audiência de outro client ID
Então o sistema responde `401 INVALID_PROVIDER_TOKEN`
Quando recebe um `idToken` válido com `email_verified` falso
Então o sistema responde `403 UNVERIFIED_PROVIDER_EMAIL` e nada é gravado
Quando a mesma rota é chamada sem `Clinicore-Client: mobile`
Então o sistema responde `400 INVALID_CLIENT`
```

### Cenário 10 — Cadastrar uma passkey (caminho feliz, regra 6)

```gherkin
Dado um usuário logado sem passkeys
Quando ele pede `POST /passkey-registration-challenges`
E envia `POST /passkeys` com a resposta do autenticador de teste para esse desafio
Então o sistema responde `201` com `id`, `createdAt` e `lastUsedAt` nulo
E `GET /passkeys` lista essa passkey
E reenviar a mesma resposta responde `400 INVALID_PASSKEY_REGISTRATION`, porque o desafio foi consumido
```

### Cenário 11 — O teto de 10 passkeys (exceção, regra 6)

```gherkin
Dado um usuário com 10 passkeys
Quando ele pede `POST /passkey-registration-challenges`
Então o sistema responde `409 PASSKEY_LIMIT_REACHED`
E dois `POST /passkeys` simultâneos com 9 passkeys cadastradas terminam com exatamente 10
```

### Cenário 12 — Entrar com passkey sem digitar o e-mail (caminho feliz, regra 7)

```gherkin
Dado um usuário com uma passkey cadastrada
Quando é pedido `POST /passkey-authentication-challenges`
E é enviado `POST /passkey-assertions` com a asserção assinada para esse desafio e com o header do app
Então o sistema responde `201` com `user` e `tokens`
E `lastUsedAt` da passkey é preenchido e o `counter` é atualizado
E sem o header a resposta traz os dois `Set-Cookie` no lugar de `tokens`
```

### Cenário 13 — Asserção recusada não revela nada (exceção, regra 7)

```gherkin
Quando `POST /passkey-assertions` recebe uma credencial que não existe
Então o sistema responde `401 INVALID_PASSKEY_ASSERTION`
E a mesma resposta vem para assinatura inválida e para desafio já usado
E nenhuma `session` é criada
```

### Cenário 14 — Remover passkey não derruba sessão (caminho feliz e exceção, regra 7)

```gherkin
Dado um usuário logado com uma passkey
Quando ele envia `DELETE /passkeys/{id}`
Então o sistema responde `204` e a sessão atual continua valendo
E remover a passkey de outro usuário responde `404 PASSKEY_NOT_FOUND`
E redefinir a senha não apaga nenhuma passkey
```

### Cenário 15 — Ambiente sem `PASSKEY_ANDROID_ORIGINS` não sobe (exceção, regra 9)

```gherkin
Dado o ambiente completo sem `PASSKEY_ANDROID_ORIGINS`
Quando a API inicia
Então o boot falha nomeando `PASSKEY_ANDROID_ORIGINS` e o formato esperado
E um item sem o prefixo `android:apk-key-hash:` também derruba o boot
```

### Cenário 16 — Nada de token no log (caminho feliz, regra 10)

```gherkin
Quando `POST /sessions/current/tokens` é chamado com `{ refreshToken }` e `POST /oauth/google/sessions` com `{ idToken }`
Então nenhuma linha do log contém o valor do `refreshToken`, do `idToken` nem do header `Authorization`
```

### Cenário 17 — Os limites novos por IP (exceção, regra 8)

```gherkin
Dado o mesmo IP
Quando `POST /passkey-authentication-challenges` é chamado 31 vezes em 60 segundos
Então a 31ª responde `429 RATE_LIMITED`
E a 11ª chamada a `POST /passkey-assertions` e a `POST /oauth/google/sessions` na mesma janela também responde `429`
```

---

## Fora de Escopo

- **As telas** — spec irmã `004-autenticacao-mobile.md`.
- **Passkey no app do iOS** — exige Associated Domains, que só existe na conta paga do Apple Developer
  Program, e o identificador definitivo do app. Entra junto da publicação, depois do nome comercial
  (`OQ-01` de `docs/requirements/open-questions.md`). Até lá, `PASSKEY_ANDROID_ORIGINS` é a única origem
  nativa aceita.
- **Links do e-mail abrindo o app** — continuam abrindo o web; o botão "Abrir o app" entra com a
  publicação.
- **Nome da passkey e detecção de aparelho** — a lista mostra só as datas; não pedidos.
- **Sessões ativas por aparelho, revogar sessão específica, 2FA** — continuam fora, como na 003.
- **Sessão do web em 7 dias** — decidido manter 24 horas (regra 3).

## Quebra em Tasks

| # | Issue | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- | --- |
| 1 | #93 | Issue the session as tokens to the mobile app | `@SessionClient()` e o enum; extração do JWT por transporte no `jwt.strategy.ts`; refresh pelo corpo; respostas com `tokens`; migration `AddSessionClient` e o tempo por cliente em `createSession` e na rotação; a nova regra da `OriginGuard`; o `redact` novo; OpenAPI das respostas | Cenários 1 a 7 e 16 verdes; os gates da API saem com código 0 | — |
| 2 | #94 | Sign in with a Google ID token from the mobile app | `google-auth-library` 11.1.0; `POST /oauth/google/sessions`; reuso do serviço de vínculo da #68; os dois códigos novos; limite da rota | Cenários 8 e 9 verdes; a linha do Google no Cenário 17 | 1, #68 |
| 3 | #100 | Register, list and remove passkeys and sign in with one | `@simplewebauthn/server` 14.0.2; migration `AddPasskey`; as seis rotas de passkey; desafios no Redis com `GETDEL`; teto de 10; `PASSKEY_ANDROID_ORIGINS` no boot e no CI; limites da regra 8 | Cenários 10 a 15 e 17 verdes | 1 |

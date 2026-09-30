# 003 — Autenticar e manter sessão (API)

> **Status:** publicada
> **Perfil:** API
> **Módulo:** `apps/api`
> **Epic:** #1 — Plataforma
> **Issue:** #3, reescrita para a stack de #72
> **Spec irmã:** `docs/specs/autenticacao/003-autenticacao-web.md` (perfil UI) — **desatualizada**, ver
> Fora de Escopo

## Acceptance Criteria

### Contrato

Todas as rotas são escritas nesta entrega. Não existe handler de terceiro montado: método, caminho,
corpo, status e cookie são os desta spec.

| Método | Rota | Auth / Role | Idempotente |
| --- | --- | --- | --- |
| `POST` | `/users` | público | Sim |
| `POST` | `/sessions` | público | Não — limitado pela regra 16 |
| `POST` | `/sessions/current/tokens` | cookie `clinicore_refresh` | Não — rotaciona (regra 3) |
| `DELETE` | `/sessions/current` | cookie `clinicore_access` | Sim |
| `GET` | `/sessions/current` | cookie `clinicore_access` | Sim |
| `POST` | `/email-verifications` | público | Sim, na janela da regra 15 |
| `POST` | `/email-verifications/confirmation` | token no corpo | Sim |
| `POST` | `/password-resets` | público | Sim, na janela da regra 15 |
| `POST` | `/password-resets/confirmation` | token no corpo | Sim |
| `PUT` | `/users/me/password` | cookie `clinicore_access` | Sim |
| `GET` | `/oauth/google` | público | Sim |
| `GET` | `/oauth/google/callback` | `state` e `code` do Google | Sim |
| `GET` | `/health` | público | Sim |

**Critério de idempotência:** uma rota é idempotente quando N chamadas iguais deixam o banco, e o que
sai por e-mail, no mesmo estado que uma chamada. A resposta pode mudar — uma segunda chamada que
responde `400` sem gravar nada continua idempotente. A contagem do limite por IP não entra no critério:
vive no Redis e é o mecanismo que limita as chamadas.

| Rota | Por que é idempotente, ou como é limitada |
| --- | --- |
| `sign-up` | O e-mail é único. Repetir responde `202` com corpo vazio, sem gravar e sem enviar (regra 8) |
| `sign-in` | **Não é.** Cada login com sucesso cria uma `session` — esse é o propósito da rota. O crescimento é limitado a 5 sessões por usuário (regra 16) |
| `refresh` | **Não é.** Cada chamada rotaciona o refresh token; a segunda chamada com o token antigo derruba a sessão (regra 3) |
| `sign-out` | Depois da primeira, a sessão não existe. Repetir responde `401` e não altera nada |
| `send-verification-email` | Dentro de 60 segundos, a repetição para o mesmo endereço não envia (regra 15) |
| `verify-email` | Depois da primeira, o token está consumido. Repetir responde `400` com `type` `invalid-token` |
| `request-password-reset` | Dentro de 60 segundos, a repetição para o mesmo endereço não grava token nem envia (regra 15) |
| `reset-password` | O token é consumido na primeira. Repetir responde `400 INVALID_TOKEN` e a senha continua a da primeira |
| `change-password` | Repetir falha em `currentPassword`, porque a senha já mudou |
| `google` | Só gera a URL de autorização e o cookie de `state`. Não grava nada (regra 6) |
| `google/callback` | O `code` do Google é de uso único. Repetir falha na troca do código e não cria sessão |

### Request

Todo corpo é JSON e é uma classe DTO com decorators de `class-validator` em
`features/auth/dto/`. **A API não aceita URL de redirecionamento do cliente em nenhuma rota**: todo
destino é montado no servidor a partir de `APP_ORIGIN` (regra 1). Campo desconhecido no corpo é
recusado.

**`POST /users`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `name` | `string` | Sim | 1 a 100 caracteres, após remover espaços das pontas; só letras, espaço, ponto, hífen e apóstrofo, com ao menos uma letra (`code` `name`). O e-mail de verificação cumprimenta só pelo primeiro nome, até 30 caracteres |
| `email` | `string` | Sim | endereço de e-mail válido; gravado em minúsculas |
| `password` | `string` | Sim | regra 4 (política de senha) |

```json
{ "name": "Ana Souza", "email": "ana@exemplo.com", "password": "Clinica#2026" }
```

**`POST /sessions`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `email` | `string` | Sim | endereço de e-mail válido |
| `password` | `string` | Sim | 1 a 128 caracteres |

**`POST /email-verifications`** e **`POST /password-resets`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `email` | `string` | Sim | endereço de e-mail válido |

**`POST /email-verifications/confirmation`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `token` | `string` | Sim | 43 caracteres base64url |

**`POST /password-resets/confirmation`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `token` | `string` | Sim | 43 caracteres base64url |
| `newPassword` | `string` | Sim | regra 4 (política de senha) |

**`PUT /users/me/password`**

| Campo | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- |
| `currentPassword` | `string` | Sim | 1 a 128 caracteres |
| `newPassword` | `string` | Sim | regra 4 (política de senha) |

**`POST /sessions/current/tokens`**, **`DELETE /sessions/current`**, **`GET /sessions/current`**, **`GET /oauth/google`**:
sem corpo e sem parâmetro. O que identifica quem chama é o cookie (regra 2).

**`GET /oauth/google/callback`**

| Parâmetro | Onde | Tipo | Obrigatório | Validação |
| --- | --- | --- | --- | --- |
| `code` | query | `string` | Sim | não vazio |
| `state` | query | `string` | Sim | igual ao cookie `clinicore_oauth_state` (regra 6) |

### Response

**`201 Created` — `POST /sessions`**, com `Location: /sessions/current`, e **`200 OK` — `GET /sessions/current`**

```json
{
  "user": {
    "id": "0b8f2c1e-5a4d-4c3b-9e7f-1a2b3c4d5e6f",
    "name": "Ana Souza",
    "email": "ana@exemplo.com",
    "emailVerified": true,
    "image": null
  }
}
```

Em `sign-in`, junto vêm os dois `Set-Cookie` da regra 2. Em `session`, nenhum cookie é reescrito.
Todo `id` é um UUID em texto (regra 12).

**`202 Accepted` com corpo vazio** — `POST /users`, `POST /email-verifications` e
`POST /password-resets`. **Sempre a mesma resposta**, para e-mail novo, já cadastrado,
inexistente, já verificado, dentro ou fora da janela da regra 15. Não há corpo porque não há nada que
possa ser dito sem revelar o estado da conta (regra 8).

**`204 No Content`** — `POST /sessions/current/tokens`, `DELETE /sessions/current`,
`POST /email-verifications/confirmation`, `POST /password-resets/confirmation` e `PUT /users/me/password`. Em `refresh` vêm os dois `Set-Cookie` novos; em `sign-out`, os dois
`Set-Cookie` de expiração.

**`302 Found`** — `GET /oauth/google` redireciona para a URL de autorização do Google, e
`GET /oauth/google/callback` redireciona para `${APP_ORIGIN}/app` com os dois cookies de sessão, ou para
`${APP_ORIGIN}/login?error=<code>`. **Os dois destinos são montados a partir de `APP_ORIGIN`**, nunca
recebidos do cliente, e nunca a partir de `ALLOWED_ORIGINS` — a landing não é destino de
autenticação.

**Corpo de erro**, em toda rota, vindo do `ExceptionFilter` global: Problem Details da RFC 9457, com
`Content-Type: application/problem+json` (ADR 0006, regra 10):

```json
{ "type": "tag:clinicore.com.br,2026:invalid-credentials", "title": "Invalid email or password", "status": 401 }
```

| Status | Quando |
| --- | --- |
| `200` | Leitura da sessão |
| `201` | Login bem-sucedido |
| `202` | As três rotas que não revelam estado de conta (regra 8) |
| `204` | Refresh, logout, confirmação de e-mail e as duas trocas de senha |
| `302` | As duas pontas do fluxo do Google |
| `400` | Corpo ou query inválidos, senha fora da política, token inválido ou consumido, senha atual incorreta |
| `401` | Credenciais incorretas, cookie de acesso ausente, expirado ou revogado, refresh inválido |
| `403` | E-mail ainda não verificado, ou `Origin` fora de `ALLOWED_ORIGINS` |
| `429` | Limite de requisições da rota excedido (regra 11) |
| `500` | Erro desconhecido, sem detalhe no corpo (regra 10) |
| `503` | Redis inalcançável (regras 2 e 11) |

### Perfis e privilégios

| Papel | Permissão | Observação |
| --- | --- | --- |
| Visitante sem sessão | — | Cadastra-se, entra, pede recuperação de senha e verifica e-mail |
| Usuário autenticado | cookie de acesso válido | Lê a própria sessão, renova, troca a própria senha e sai |
| Orquestrador de container | — | `GET /health`, antes de existir sessão |

**Não existe papel nesta entrega.** O usuário autenticado não pertence a nenhuma clínica e não tem
função. Papéis, permissões e tenancy são as issues #6 e #7.

---

## Regras de Negócio

### 1. O browser fala com a API em origem cruzada, com credenciais, e o `Origin` é conferido

- **Duas variáveis, dois papéis.** `APP_ORIGIN` é o sistema, `https://app.clinicore.com.br`, e é a única
  origem a partir da qual a API monta destino de redirecionamento. `ALLOWED_ORIGINS` é a lista de
  origens que podem falar com a API pelo browser: o sistema, e a landing `https://clinicore.com.br`
  quando ela passar a chamar alguma rota pública. Em desenvolvimento as duas valem
  `http://localhost:3000`.
- `main.ts` chama `app.enableCors({ origin: env.ALLOWED_ORIGINS, credentials: true, methods: ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"], allowedHeaders: ["Content-Type"] })`
  — a **lista de origens exatas**, nunca `*` e nunca um curinga. O Express responde com a origem que
  pediu, quando ela está na lista.
- **O CORS não é a guarda.** Ele instrui o browser; não impede requisição alguma de chegar. A guarda é
  `common/guards/origin.guard.ts`, registrada como `APP_GUARD`: **todo método que não é `GET`, `HEAD`
  nem `OPTIONS` precisa do header `Origin` pertencente a `ALLOWED_ORIGINS`**, comparado por igualdade exata contra cada item da lista, e
  qualquer outro valor, ou a ausência do header, responde `403 INVALID_ORIGIN` antes de o controller
  rodar. **Comparação é de origem inteira, nunca de sufixo** — casar o fim da string aceitaria
  `https://clinicore.com.br.evil.example`.
- **Por que a guarda existe, mesmo com o cookie em `SameSite=Lax`:** `Lax` já não acompanha um `POST`, `PUT`,
  `PATCH` ou `DELETE` vindo de outro site, então ela deixou de ser a única linha contra CSRF e passou a ser a segunda. Ela
  continua porque `/sessions/current/tokens` e `/sessions/current` não têm corpo — nem a validação de DTO os
  protegeria — e porque é ela que faz a API recusar cedo, no lugar de depender de o browser ter se
  comportado.
- `GET`, `HEAD` e `OPTIONS` são isentos: `/health`, `/sessions/current` e as duas rotas do Google não
  mudam estado a partir de um corpo, e a navegação de volta do Google chega sem `Origin`.
- **Não existe redirecionamento aberto porque não existe parâmetro de redirecionamento.** Nenhuma rota
  lê `callbackURL`, `redirectTo` ou `errorCallbackURL`; o DTO recusa campo desconhecido, e os dois
  destinos da API são montados no servidor a partir de `APP_ORIGIN`. **Um destino nunca sai de
  `ALLOWED_ORIGINS`:** a lista existe para autorizar quem chama, não para escolher para onde mandar o
  usuário — e é isso que impede que incluir a landing um dia vire um redirecionamento aberto.
- `app.set("trust proxy", env.TRUSTED_PROXIES)` é aplicado no adapter do Express antes do listen, e é o
  que faz `req.ip` valer (regra 11).

### 2. A sessão são dois cookies: um JWT de acesso curto e um refresh opaco

A sessão não é um token único. São dois, com tempos de vida e caminhos diferentes:

| Cookie | Conteúdo | `Path` | `Max-Age` | Quem valida |
| --- | --- | --- | --- | --- |
| `clinicore_access` | JWT assinado com `JWT_SECRET` | `/` | 900 (15 min) | `JwtStrategy`, sem tocar o banco |
| `clinicore_refresh` | `<sessionId>.<segredo>` opaco | `/sessions/current/tokens` | 86400 (24 h) | `POST /sessions/current/tokens`, contra a tabela `session` |

- Os dois são **`HttpOnly: true` sempre**. O JavaScript do web nunca lê nenhum dos dois; quem carrega o
  usuário é `GET /sessions/current` com `credentials: "include"`.
- **`sameSite: "lax"` em todo ambiente.** `app.clinicore.com.br` e `api.clinicore.com.br` têm o mesmo
  domínio registrável, `clinicore.com.br`, e `SameSite` é calculado por **domínio registrável, não por
  origem**: são o mesmo site, e o cookie `Lax` acompanha tanto a chamada do sistema quanto a navegação de
  volta do Google, que é um `GET` de topo. Em desenvolvimento vale o mesmo raciocínio por outro caminho —
  `localhost:3000` e `localhost:3333` são o mesmo site, porque a porta não conta para `SameSite`.
- **`secure: true` quando `NODE_ENV === "production"`**, e `false` em desenvolvimento, porque
  `http://localhost` não aceita `Secure`.
- **`SameSite=None` seria um erro, não uma necessidade.** Ele é o que faria o cookie acompanhar um `POST`
  disparado de qualquer outro site; com `Lax` essa porta está fechada no browser, e a guarda de `Origin`
  da regra 1 passa a ser a segunda linha em vez da única. Hospedar a API fora de `clinicore.com.br` — num
  domínio de plataforma, por exemplo — quebraria essa premissa e obrigaria a rever esta regra inteira.
- **O `Path` do refresh é `/sessions/current/tokens`, e isso é o ponto.** O refresh token não acompanha nenhuma
  outra requisição. `DELETE /sessions/current` não precisa dele: o `sessionId` está no JWT de acesso.
- **Nenhum `Domain`.** Os cookies pertencem ao host da API e não são compartilhados com subdomínio
  nenhum. É o que impede a landing, que é pública e fica no ápice do domínio, de receber cookie de
  sessão.
- O JWT de acesso carrega `sub` (o `user.id`), `sid` (o `session.id`) e `exp`. Nada mais — nome,
  e-mail e `emailVerified` saem de `GET /sessions/current`, para que uma mudança neles não fique presa no
  token por 15 minutos.
- **A denylist do Redis é o que torna a revogação imediata.** Revogar uma sessão grava
  `auth:revoked:<sessionId>` com TTL de 900 segundos — o tempo de vida do access token. O
  `JwtStrategy` consulta essa chave em toda requisição autenticada e responde `401 INVALID_SESSION` se
  ela existir. Passados os 900 segundos, nenhum token daquela sessão pode mais existir e a chave some
  sozinha.
- **Toda revogação escreve na denylist**, sem exceção: logout, reset de senha, troca de senha, reuso de
  refresh detectado e corte pelo teto de 5 sessões (regra 16).
- **O Redis é dependência dura da requisição autenticada, e a falha é fechada.** Redis inalcançável faz
  o `JwtStrategy` responder `503` com o código `SERVICE_UNAVAILABLE`, e isso vale também para a rota
  que revoga: ela é autenticada, então a consulta à denylist acontece antes do service e nada é
  revogado pela metade — a linha em `session` sobrevive. Responder `204` num logout cujo token continua
  valendo por 15 minutos é pior do que responder erro.

### 3. O refresh rotaciona a cada uso, e reusar o antigo derruba a sessão

- O refresh token é `<sessionId>.<segredo>`, com o segredo em 32 bytes aleatórios em base64url. A
  tabela `session` guarda **o SHA-256 do segredo**, nunca o segredo.
- **SHA-256, não argon2.** O segredo tem 256 bits de entropia e não é adivinhável por força bruta; o
  argon2 existe para senha escolhida por gente (regra 4). A comparação é feita com
  `crypto.timingSafeEqual`.
- `POST /sessions/current/tokens` faz, numa única transação de repository:
  1. localiza a `session` pelo `sessionId` do token;
  2. compara o hash do segredo recebido com o gravado;
  3. **iguais:** grava um segredo novo, empurra `expiresAt` para 24 horas à frente, devolve `204` com os
     dois cookies novos;
  4. **diferentes:** apaga a `session`, grava o `sessionId` na denylist e responde
     `401 SESSION_REUSED`.
- **O caso 4 é a detecção de reuso.** Um segredo que não bate é ou um token roubado e já rotacionado, ou
  uma forja. Nos dois casos a sessão está comprometida e morre inteira, junto com o access token que
  ainda estiver vivo.
- Sessão inexistente ou vencida responde `401 INVALID_SESSION`, sem gravar nada.
- **A sessão dura 24 horas de inatividade**, empurradas a cada rotação, e **nunca mais de 30 dias desde
  o `createdAt`**. Passado o teto, o refresh responde `401 INVALID_SESSION` como uma sessão vencida: um
  refresh roubado que rotaciona à frente da vítima não vive para sempre. **Não existe "lembrar de
  mim"**: toda sessão dura o mesmo.
- **O refresh só vale pelo transporte em que a sessão nasceu.** Um refresh de sessão `mobile`
  apresentado no cookie, ou o contrário, responde `401 INVALID_SESSION` sem apagar a sessão.
- **Não existe conceito de sessão fresca.** A única operação sensível desta entrega é
  `change-password`, que já exige `currentPassword`.
- Duas chamadas simultâneas de `/sessions/current/tokens` com o mesmo token válido: a transação serializa, uma
  rotaciona e a outra cai no caso 4 e derruba a sessão. É o comportamento correto — o custo é um
  relogin, e o benefício é que roubo de refresh não passa despercebido. O web serializa o refresh numa
  única chamada em voo.

### 4. A política de senha

Uma senha é aceita quando cumpre **todas** as condições:

- no mínimo 8 e no máximo 128 caracteres;
- ao menos uma letra maiúscula;
- ao menos um dígito;
- ao menos um caractere que não seja letra nem dígito.

- A regra vive em `features/auth/utils/password-policy.ts` como **função pura, testada**, exportando
  `isStrongPassword(password: string): boolean`, que confere as quatro condições, tamanho incluído.
- A aplicação é um decorator `@IsStrongPassword()` de `class-validator`, montado com `registerDecorator`
  sobre essa função, usado nos DTOs de `sign-up`, `reset-password` e `change-password`. **A guarda fica
  no DTO, não em cada service.**
- O `ValidationPipe` global recusa com `400 VALIDATION_FAILED` e
  `errors: [{ "pointer": "#/password", "code": "weak_password" }]`, ou `#/newPassword` (regra 10).
- A senha é hasheada com `@node-rs/argon2`, algoritmo `Argon2id`, nos parâmetros padrão da biblioteca.
  O hash vive em `account.passwordHash` e nunca em `user`.

### 5. O e-mail é verificado antes do primeiro login

- `POST /users` **não cria sessão** e não devolve cookie. Responde `202` e envia o link.
- `POST /sessions` com a senha correta e `user.emailVerified = false` responde
  `403 EMAIL_NOT_VERIFIED`, não cria sessão, e **reenvia o link**, sujeito à regra 15. Senha errada
  responde `401 INVALID_CREDENTIALS` antes desse ponto, sem enviar nada.
- O token de verificação é uma linha em `verification` com `purpose = "email_verification"`, 32 bytes
  aleatórios em base64url, gravados como SHA-256, com `expiresAt` 1 hora à frente.
- **O link do e-mail aponta para o web**, `${APP_ORIGIN}/verify-email?token=<token>`, e é a página do
  web que chama `POST /email-verifications/confirmation` com `{ token }` no corpo (ADR 0006). O link
  nunca aponta para a API: um `GET` que consome token seria gasto pelo scanner de link do cliente de
  e-mail antes da pessoa clicar.
- `POST /email-verifications/confirmation` consome o token:
  - válido → marca `user.emailVerified = true`, grava `verification.consumedAt` e responde `204`
    **sem sessão e sem cookie**; a pessoa entra depois com a senha que escolheu no cadastro;
  - inválido, consumido ou inexistente → `400 INVALID_TOKEN`, sem cookie;
  - expirado → `400 TOKEN_EXPIRED`, sem cookie.
- Uma conta criada pelo Google nasce com `emailVerified = true` e nunca passa por esta regra (regra 6).
- **A confirmação não abre sessão porque o cadastro não prova a posse do endereço.** Quem cadastra o
  e-mail de outra pessoa escolhe a senha; se clicar no link logasse a vítima, ela usaria a conta
  enquanto o atacante guarda a senha. Sem sessão, a dona do endereço só entra redefinindo a senha
  (regra 7), o que troca a senha e derruba toda sessão.
- Ao marcar `emailVerified = true`, **todos os tokens de verificação pendentes daquele endereço são
  consumidos na mesma transação**, e dois links do mesmo endereço confirmados ao mesmo tempo são
  serializados por endereço: um responde `204` e o outro `400 INVALID_TOKEN`.

### 6. Google e senha são a mesma conta

- `GET /oauth/google` monta a URL de autorização do Google e responde `302` para ela, com
  `prompt=select_account` e `scope=openid email profile`. **Não grava nada no banco** — o `state` viaja
  em cookie.
- **O `state` é um cookie, não uma linha.** `clinicore_oauth_state` guarda 32 bytes aleatórios em
  base64url, `HttpOnly`, `Path=/oauth/google`, `Max-Age=600`, `SameSite=Lax`, e `Secure` em produção.
  `Lax` basta porque a volta do Google é uma navegação `GET` de topo, que carrega cookie `Lax`. O
  `state` da query é comparado com o do cookie em `crypto.timingSafeEqual`; diferente ou ausente,
  `302` para `${APP_ORIGIN}/login?error=INVALID_STATE`.
- A implementação é `@nestjs/passport` com `passport-google-oauth20`, e o `state` é guardado por um
  `store` próprio sobre o cookie — `passport-oauth2` aceita um `store` e é isso que dispensa
  `express-session`. A URL de callback registrada no Google Cloud Console é
  `${API_URL}/oauth/google/callback`.
- **O perfil do Google só é aceito com `email_verified = true`.** Falso, a resposta é `302` para
  `${APP_ORIGIN}/login?error=UNVERIFIED_PROVIDER_EMAIL`, e nada é gravado. Sem isso, um provedor que
  devolvesse um e-mail não verificado sequestraria a conta de quem tem esse endereço.
- Entrar com Google num e-mail que já tem cadastro por senha **vincula** o provedor à conta existente:
  nasce uma linha em `account` com `provider = "google"` apontando para o mesmo `userId`, e nenhum
  `user` novo é criado. Entrar com a senha original continua funcionando.
- E-mail sem cadastro nenhum: `user` e `account` são criados na mesma transação, com
  `emailVerified = true`.
- **Nenhum token do Google é guardado.** O `access_token` e o `refresh_token` da troca são descartados
  depois de lido o perfil; a API não chama API nenhuma do Google depois do login. Por isso a tabela
  `account` não tem coluna de token.
- O callback cria a sessão, aplica a regra 16 e responde `302` para `${APP_ORIGIN}/app` com os dois
  cookies.

### 7. Recuperar e trocar senha

- `POST /password-resets` grava uma linha em `verification` com
  `purpose = "password_reset"`, token de 32 bytes aleatórios em base64url gravado como SHA-256 e
  `expiresAt` 1 hora à frente, e envia o link, sujeito à regra 15. Responde `202` sempre (regra 8).
- **O link aponta direto para o web**, `${APP_ORIGIN}/reset-password?token=<token>`. Não existe rota de
  API que apenas redirecione: ela só ampliaria a superfície de redirecionamento sem fazer nada.
- `POST /password-resets/confirmation` faz, numa única transação de repository: consome o token, grava o hash da
  nova senha em `account` e **apaga todas as sessões do usuário**. Fora da transação, cada `sessionId`
  apagado vai para a denylist (regra 2). Responde `204`.
- **Redefinir a senha derrubar todas as sessões é imediato**, inclusive os access tokens ainda dentro
  dos 15 minutos, por causa da denylist. Sem ela a regra seria falsa por até 15 minutos.
- Duas chamadas simultâneas com o mesmo token: a primeira troca, a segunda responde
  `400 INVALID_TOKEN`, porque o token é consumido dentro da transação.
- **Redefinir a senha de um usuário sem `account` de senha cria essa `account`.** É assim que quem
  entrou só com Google passa a ter também login por senha.
- `PUT /users/me/password` confere `currentPassword` com o argon2, grava o hash novo e **apaga
  todas as sessões do usuário exceto a do cookie usado na requisição**, mandando as apagadas para a
  denylist. Quem troca a senha continua logado no dispositivo atual e cai em todos os outros. Não há
  parâmetro para desligar isso. Responde `204`; o cookie de acesso e o de refresh atuais continuam
  valendo.

### 8. Nenhuma resposta revela quem tem conta

- **As três rotas que recebem um e-mail sem sessão respondem `202` com corpo vazio, sempre.**
  `sign-up`, `send-verification-email` e `request-password-reset` respondem igual para e-mail novo, já
  cadastrado, inexistente, já verificado, dentro e fora da janela da regra 15. Não há corpo, então não
  há nada que possa diferir.
- `POST /sessions` responde `401 INVALID_CREDENTIALS` tanto para e-mail inexistente quanto para
  senha errada.
- **O tempo de resposta também não diferencia.** Quando não existe `account` de senha para o e-mail, o
  service verifica a senha recebida contra um **hash argon2 fixo, gerado no boot**, e descarta o
  resultado. Sem isso, o login de um e-mail inexistente responderia sem gastar o tempo do argon2 e a
  diferença seria medível.
- **Cadastro concorrente não vaza.** Dois `sign-up` simultâneos com o mesmo e-mail novo: o segundo
  falha no índice único de `user.email`, o repository traduz o `QueryFailedError` do código `23505`
  para o tipo `Conflict`, e o service **engole esse conflito e responde `202` igual aos outros**. Nada
  é gravado e nada é enviado.
- **Nenhum e-mail avisa o dono do endereço** de que alguém tentou cadastrar com ele: seria um vetor de
  envio sem limite por endereço.

### 9. Todo request é logado, e nenhum segredo sai no log

- `core/logger/` cria a instância do Pino 10 e o `LoggerService` do Nest registrado por
  `app.useLogger()`, mais o **middleware** que registra a requisição. **Nenhum wrapper de terceiro.**
- **É middleware, e não interceptor, porque o guard roda antes do interceptor.** Um interceptor não
  veria o `403 INVALID_ORIGIN` da guarda da regra 1 nem o `404` de rota inexistente, e a regra
  valeria só para o request que alcança um controller. O middleware registra o `res.on("close")`,
  então enxerga o status final venha ele do controller, do guard ou do filtro.
- O nível vem de `LOG_LEVEL`.
- Cada requisição registra método, caminho, status e duração em milissegundos.
- **`GET /health` não é logado.**
- O `redact` do Pino cobre, no mínimo: `req.headers.cookie`, `req.headers.authorization`,
  `res.headers["set-cookie"]`, `req.body.password`, `req.body.newPassword`, `req.body.currentPassword`
  e `req.body.token`.
- **O `Environment` nunca é passado ao logger.** A connection string, o `JWT_SECRET` e os segredos de
  SMTP e do Google não entram em nenhum objeto logado, então não existe caminho de `redact` para eles.
- O Pino escreve em `stdout`. O erro de ambiente do boot continua indo cru para `stderr`, antes de
  existir logger (regra 14).
- **Em `development` o destino é o `pino-pretty`**, colorido e legível; em `test` e em `production` é
  `stdout` cru, uma linha JSON por evento. Quem decide é o `NODE_ENV`, não uma variável nova, e o
  `pino-pretty` é dependência de desenvolvimento — a imagem de produção não o instala.
- O worker da regra 17 usa a mesma instância de logger.

### 10. O erro tem um catálogo próprio, e o desconhecido vira 500 sem detalhe

- O service lança `BusinessError` de `common/exceptions/`, com um tipo (`NotFound`, `Conflict`,
  `Forbidden`, `Invalid`, `Unauthorized`) e um código do catálogo da seção **Erros**.
- O repository traduz o erro conhecido do TypeORM — `QueryFailedError` com o código do Postgres,
  `EntityNotFoundError` — para esses tipos, e nunca deixa vazar erro de driver.
- `common/filters/business-error.filter.ts`, registrado como `APP_FILTER`, é **o único lugar do app que
  conhece HTTP**. Converte o tipo em status e responde Problem Details da RFC 9457 (ADR 0006), com
  `Content-Type: application/problem+json`:
  - `type` é `tag:clinicore.com.br,2026:` seguido do código do catálogo em minúsculas e com hífen —
    `INVALID_SESSION` sai como `tag:clinicore.com.br,2026:invalid-session`. **O cliente decide pelo
    `type`**, nunca pelo `title`;
  - `title` é a mensagem do catálogo, fixa para cada `type`;
  - `status` repete o status HTTP;
  - `errors` só existe na validação; `detail` e `instance` não existem.
- O `ValidationPipe` global roda com `whitelist: true`, `forbidNonWhitelisted: true` e
  `transform: true`, e um `exceptionFactory` que lança `VALIDATION_FAILED` com `errors`: um item por
  campo, com `pointer` em JSON Pointer (`#/email`, `#/address/zip`) e `code` com o **nome da primeira
  restrição violada**, em maiúsculas com sublinhado:
  `[{ "pointer": "#/email", "code": "email" }, { "pointer": "#/password", "code": "weak_password" }]`.
  O texto que a pessoa lê é escrito no web a partir do `type` e desses códigos, nunca do `title`.
- **Um `HttpException` do próprio Nest mantém o status e usa `about:blank`.** Rota inexistente
  continua `404`, método errado continua `405`, e o `title` é a frase padrão do status
  (`http.STATUS_CODES`) — `"Not Found"`, `"Method Not Allowed"` —, como a RFC pede para `about:blank`.
  A mensagem gerada pelo Nest nunca chega ao corpo. É regra geral, não uma tabela por caso.
- **Um `HttpException` 5xx mantém o status, e é registrado no Pino com a stack.** O corpo é o mesmo
  `about:blank` com a frase do status, então o texto da exceção nunca vaza.
- Um erro que não é `BusinessError` nem `HttpException` é registrado no Pino com a stack completa e
  respondido como `500` com corpo
  `{ "type": "about:blank", "title": "Internal Server Error", "status": 500 }`.
- O `title` é inglês e é texto de desenvolvedor, para log e depuração.

### 11. Toda rota de autenticação tem limite por IP, e o IP não pode ser forjado

- `@nestjs/throttler` é registrado em `core/redis/` com `@nest-lab/throttler-storage-redis`, a storage
  que a documentação do próprio `@nestjs/throttler` indica, sobre o mesmo cliente `ioredis` da fila e
  da denylist. **O `ioredis` é dependência direta do `package.json`**, não transitiva: o
  `@nest-lab/throttler-storage-redis` o declara como peer, e o `bullmq` 6 o declara como peer
  **opcional** — ninguém o instala sozinho, e a falta só aparece em runtime. **A contagem vive no
  Redis, não em tabela.** Uma linha de contagem por requisição num
  banco relacional é escrita de escrita alta com TTL, que é exatamente o que o Redis faz e o Postgres
  não.
- `ThrottlerGuard` é registrado como `APP_GUARD`; o limite por rota é um `@Throttle()` no controller:

  | Caminho | Janela | Máximo |
  | --- | --- | --- |
  | `/users` | 60 s | 3 |
  | `/sessions` | 60 s | 5 |
  | `/sessions/current/tokens` | 60 s | 30 |
  | `/email-verifications` | 60 s | 3 |
  | `/password-resets` | 60 s | 5 |
  | `/password-resets/confirmation` | 60 s | 5 |
  | `/users/me/password` | 60 s | 3 |
  | `/oauth/google` e `/oauth/google/callback` | 60 s | 10 |
  | qualquer outra rota | 10 s | 100 |
  | `/health` | sem limite, por `@SkipThrottle()` | — |

- **Nenhuma rota é limitada por uma chave só.** O IP é trocável — um bloco IPv6 barato dá dezenas de
  milhares de `/64` —, então toda rota limitada conta também numa segunda chave, independente do IP:

  | Rota | Segunda chave | Janela | Máximo |
  | --- | --- | --- | --- |
  | `/sessions` | falhas de senha por e-mail, em minúsculas | 15 min | 10 |
  | rotas com sessão | a sessão do access token, por rota | 10 s | 100 |
  | `/sessions/current/tokens` | a sessão do refresh token | 60 s | 30 |
  | `/email-verifications/confirmation` | a faixa de rede do IP: /48 no IPv6, /24 no IPv4 | 60 s | 300 |
  | `/users`, `/email-verifications`, `/password-resets` | o endereço, pela regra 15 | — | — |

- **A confirmação é a única segunda chave que sai do IP.** O corpo dela só traz o token, e o e-mail só
  aparece quando o token é válido, então nenhuma identidade conta um palpite. A faixa de rede pega quem
  troca de endereço dentro da mesma rede, e um teto total da rota deixaria um cliente só travar a
  confirmação de todo mundo.

- **O contador do login por e-mail conta só falha e barra antes do argon2**, inclusive com a senha
  certa. Cada tentativa soma antes de conferir a senha e devolve o ponto quando ela confere, então uma
  rajada de tentativas simultâneas não passa do teto. Travar a conta de outra pessoa por 15 minutos é
  o custo aceito: o Google e o reset continuam funcionando.
- Excedido, o `ThrottlerGuard` lança e o filtro responde `429` com o código `RATE_LIMITED`.
- **A falha é fechada, como na regra 2.** Redis inalcançável durante a contagem responde `503` com o
  código `SERVICE_UNAVAILABLE`, em vez de deixar a requisição passar sem limite. O `/health` não conta,
  então continua respondendo.
- A ordem dos guards globais é `Origin` → limite → JWT: `Origin` inválido responde `403` sem gastar
  contagem, e uma rota autenticada conta a tentativa antes de o cookie ser conferido.
- **O IP vem da cadeia de proxies confiáveis, pelo próprio Express.**
  `app.set("trust proxy", env.TRUSTED_PROXIES)` recebe a lista de blocos CIDR; o Express percorre
  `x-forwarded-for` da direita para a esquerda, pula os hops confiáveis e entrega em `req.ip` o
  primeiro que não é. `getTracker(req)` do guard devolve `req.ip` normalizado pelo próprio
  `@nestjs/throttler`: `::ffff:` removido do IPv4 mapeado, e IPv6 reduzido ao `/64`, que é a alocação
  padrão de um cliente — sem isso, trocar de endereço dentro do próprio `/64` daria um contador novo a
  cada requisição. O que o cliente escreve à esquerda
  do hop que o proxy acrescentou nunca é usado.
- Uma cadeia em que **todos** os hops são confiáveis faz o Express entregar o valor mais à esquerda,
  que o cliente controla. Por isso `TRUSTED_PROXIES` nomeia exatamente a sub-rede do proxy à frente da
  API, e nunca uma faixa maior. Sem `req.ip`, o tracker é a string fixa `"unknown"` e essas
  requisições dividem um contador só, em vez de escaparem do limite.
- **A garantia depende do deploy:** a API só pode ser alcançável através do proxy que acrescenta o hop.
  Exposta direto, um `x-forwarded-for` forjado de valor único seria aceito como IP do cliente. Isso é a
  issue #4 (Fora de Escopo).
- Em `development`, sem proxy, o IP é o do socket: `127.0.0.1` quando o cliente chama por IPv4, e
  `::1` — contado como `::/64` — quando `localhost` resolve para IPv6, como no `curl localhost`.

### 12. Persistência e Auditoria

- **Cinco tabelas, em duas migrations**, todas geradas por `typeorm migration:generate` e revisadas
  antes do commit, em `core/db/migrations/`, com `synchronize: false` em todo ambiente (regra 6 da
  spec `002`). As entities do TypeORM ficam em `features/auth/entities/`, uma classe por tabela, e
  são a única fonte do schema: índice, unique e nome de enum nascem nelas, nunca editados no arquivo
  gerado.

  | Tabela | Migration | Colunas |
  | --- | --- | --- |
  | `user` | `AddAuth` | `id`, `name`, `email` (único, minúsculas), `emailVerified` (bool, padrão `false`), `image` (nulo), `createdAt`, `updatedAt` |
  | `account` | `AddAuth` | `id`, `userId` → `user.id` `ON DELETE CASCADE`, `provider` (`credential` ou `google`), `providerAccountId` (nulo), `passwordHash` (nulo), `createdAt`, `updatedAt`; único em (`userId`, `provider`) e em (`provider`, `providerAccountId`) |
  | `session` | `AddAuth` | `id`, `userId` → `user.id` `ON DELETE CASCADE` (indexado), `refreshTokenHash` (único), `expiresAt` (indexado), `ipAddress` (nulo), `userAgent` (nulo), `createdAt`, `updatedAt` |
  | `verification` | `AddAuth` | `id`, `identifier` (o e-mail, minúsculas), `purpose` (`email_verification` ou `password_reset`), `tokenHash` (único), `expiresAt` (indexado), `consumedAt` (nulo), `createdAt`; índice composto em (`identifier`, `purpose`, `consumedAt`) |
  | `emailDispatch` | `AddEmailDispatch` | `id`, `email` (minúsculas), `kind` (`verification` ou `password_reset`), `createdAt` (indexado); índice composto em (`email`, `kind`, `createdAt`) |

- **Não existe tabela de limite de requisições.** A contagem é do Redis (regra 11).
- Os índices em `session.expiresAt`, `verification.expiresAt` e `emailDispatch.createdAt` existem para
  que a limpeza da regra 17 não varra a tabela inteira.
- **Toda chave primária é UUID**, com `@PrimaryGeneratedColumn("uuid")` na entity e
  `DEFAULT gen_random_uuid()` na migration — nativo do PostgreSQL 13 em diante, sem extensão. Quem o
  entrega é `uuidExtension: "pgcrypto"` no `data-source.options.ts`; sem essa opção o gerador escreve
  `uuid_generate_v4()`, que depende da extensão `uuid-ossp`. Ao lado dela, `installExtensions: false`
  impede o `CREATE EXTENSION` que o TypeORM roda a cada boot.
  O gate de que a entity e a migration não divergiram é um step do job da API no CI: roda
  `typeorm migration:generate` apontando para um arquivo temporário e **falha se esse arquivo for
  criado**.
- `provider`, `purpose` e `kind` são colunas de enum do Postgres, criadas pela migration e espelhadas
  em `features/auth/enums/`. **A coluna não declara `enumName`**: o nome derivado já é
  `<tabela>_<coluna>_enum`, e declará-lo explicitamente produz drift permanente.
- **Cadastro por senha e por Google são atômicos**, em `dataSource.transaction()` dentro do
  repository, como manda o contrato: `user`, `account` e, no cadastro por senha, a linha de
  `emailDispatch` da regra 15 são gravados juntos ou nenhum é. **O envio do e-mail fica fora da
  transação** (regra 13).
- São também uma transação só: a rotação do refresh (regra 3), o consumo de token com troca de senha
  (regra 7), o consumo de token com verificação de e-mail (regra 5), a checagem-e-gravação de
  `emailDispatch` (regra 15) e o corte do teto de sessões (regra 16).
- **Auditoria:** `N/A` nesta entrega. Não há acesso a prontuário para registrar, e a trilha de
  auditoria é a issue #9.
- **Eventos/integrações disparados:** envio de e-mail por SMTP (regra 13) e o job agendado de limpeza
  (regra 17).

### 13. O e-mail sai direto, sem fila, e o envio não desfaz o cadastro

- `core/mail/` cria o transport do Nodemailer sobre o SMTP do Gmail, em todos os ambientes, a partir de
  `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER` e `SMTP_PASSWORD`.
- **O `From` é o da conta autenticada, e `MAIL_FROM` só nomeia o remetente.** O Gmail reescreve o
  endereço do `From` com a conta autenticada, em silêncio; o alias do "Enviar e-mail como" não vale no
  `smtp.gmail.com`. Por isso `MAIL_FROM` carrega o nome de exibição e **precisa ser o endereço da
  conta de `SMTP_USER`**, e o boot recusa valores diferentes.
- **Isso é um trade-off declarado, não um descuido.** Até o lançamento público, o e-mail transacional
  sai de uma conta comum do Gmail, com app password e verificação em duas etapas ligada. O teto de
  envio é diário e da conta inteira, e dev e homolog o dividem. Trocar para um remetente próprio é
  `smtp-relay.gmail.com` com Google Workspace, ou outro provedor — decisão do lançamento, não desta
  entrega.
- Dois e-mails, os dois em pt-BR:

  | Gatilho | Assunto | Corpo |
  | --- | --- | --- |
  | Cadastro, login sem verificação e reenvio | "Confirme seu e-mail no Clinicore" | "Olá, {nome}. Confirme seu e-mail para começar a usar o Clinicore. O link expira em 1 hora." + botão "Confirmar e-mail" |
  | Recuperação de senha | "Redefinir sua senha do Clinicore" | "Olá, {nome}. Recebemos um pedido para redefinir sua senha. O link expira em 1 hora. Se não foi você, ignore este e-mail." + botão "Redefinir senha" |

- **O envio acontece depois da transação, e falhar nele não muda a resposta.** SMTP fora do ar registra
  um log de nível `error` com o endereço e o motivo, e a rota responde exatamente o que responderia com
  sucesso: `202`. **O motivo:** desfazer o cadastro porque o e-mail não saiu deixaria a pessoa sem conta
  e sem aviso, e responder erro revelaria o estado da conta nas três rotas da regra 8. O link é
  re-solicitável pelo botão de reenvio, que é justamente para isso.
- A linha de `emailDispatch` é gravada **antes** do envio (regra 15), então um envio que falha consome
  um dos 5 pedidos diários. É o preço de a contagem não poder depender do resultado sem revelar o
  estado da conta.
- A fila da regra 17 não é usada para e-mail.

### 14. Toda variável nova é obrigatória e validada no boot

Segue a regra 4 da spec `002`, sem exceção: classe `Environment` com decorator de `class-validator` por
variável em `core/config/env.validation.ts`, `validateEnv` puro e testado, `main.ts` escrevendo em
`stderr` e saindo com `1`, nenhum valor padrão em nenhum ambiente, o valor recebido nunca impresso.
**A API e o worker leem o mesmo módulo** e exigem o mesmo conjunto de variáveis.

| Nome | Tipo | Formato esperado |
| --- | --- | --- |
| `API_URL` | string | `expected an absolute URL with no trailing slash (https://…)` |
| `GOOGLE_CLIENT_ID` | string | `expected a non-empty string` |
| `GOOGLE_CLIENT_SECRET` | string | `expected a non-empty string` |
| `JWT_SECRET` | string | `expected a string with at least 32 characters` |
| `LOG_LEVEL` | string | `expected one of: fatal, error, warn, info, debug, trace, silent` |
| `MAIL_FROM` | string | `expected an email address equal to SMTP_USER` |
| `REDIS_URL` | string | `expected a Redis connection string (redis://…)` |
| `SMTP_HOST` | string | `expected a hostname` |
| `SMTP_PASSWORD` | string | `expected a non-empty string` |
| `SMTP_PORT` | inteiro | `expected an integer between 1 and 65535` |
| `SMTP_USER` | string | `expected an email address` |
| `TRUSTED_PROXIES` | lista de string | `expected a comma-separated list of CIDR blocks (10.0.0.0/8,…)` |

- `NODE_ENV`, `DATABASE_URL`, `PORT`, `APP_ORIGIN` e `ALLOWED_ORIGINS` já nascem na `002` e não mudam.
  `LOG_LEVEL` é declarada lá como pertencente a esta entrega, e é aqui que ela nasce.
- **`TRUSTED_PROXIES` precisa de um decorator próprio.** O `class-validator` não tem decorador de CIDR:
  ele sai de um `registerDecorator` de poucas linhas sobre `isIPRange` do `validator` 13, que o próprio
  `class-validator` já traz como dependência. Nunca de regex à mão.
- `MAIL_FROM` é validada **contra outra variável**, com um decorator que compara com `SMTP_USER`
  (regra 13). Diferente, o boot falha.
- O teste de boot da `002` passa a listar todas as variáveis ausentes, e o caso feliz precisa das novas.
- `.github/workflows/ci.yml` recebe as mesmas variáveis no bloco `env` do job `api`, um serviço
  `redis:8` ao lado do `postgres:18`, e um step `npm run migration:run` antes dos testes — o step de
  migration nasce na task #66, junto da primeira migration.

### 15. Pedidos de e-mail são limitados por endereço, e não só por IP

O limite por IP da regra 11 não protege uma vítima: de vários IPs, o mesmo endereço receberia e-mails
sem fim, cada pedido de reset gravaria uma linha em `verification`, e o teto diário da conta de envio
(regra 13) acabaria para todos os usuários.

- **Por endereço, em qualquer IP:** no máximo **1 pedido a cada 60 segundos** e **5 pedidos a cada 24
  horas**, contados separadamente para `verification` e `password_reset`.
- A contagem vive em `emailDispatch`. Cada pedido aceito grava uma linha; o pedido barrado não grava.
- A guarda é **um método de `features/auth/repository/email-dispatch.repository.ts`**,
  `registerDispatch(email, kind): boolean`, que confere as duas janelas e grava a linha **na mesma
  transação**, e devolve se o pedido passou. Os quatro caminhos que enviam e-mail — `sign-up`,
  `sign-in` sem verificação, `send-verification-email` e `request-password-reset` — chamam esse método
  antes de montar o token, e não enviam nada quando ele devolve `false`.
- **Um pedido barrado não grava token.** Nada entra em `verification`, nada sai por SMTP, e a rota
  responde exatamente como responderia enviando: `202` com corpo vazio, ou `403 EMAIL_NOT_VERIFIED` no
  caso do `sign-in`.
- **O pedido conta exista a conta ou não.** A gravação acontece antes de o service consultar o usuário.
  Se só pedidos de contas existentes fossem registrados, o comportamento sob o limite revelaria que o
  e-mail tem cadastro.
- Isso torna as três rotas da regra 8 **idempotentes dentro da janela de 60 segundos**: repetir não
  grava e não envia. Fora da janela, um novo e-mail é o comportamento pedido por quem clica em
  reenviar.
- O web mostra o botão de reenvio desabilitado por 60 segundos, mas a guarda é esta, não a do web.

### 16. Cada usuário tem no máximo 5 sessões ativas

- Toda sessão nasce em um único método,
  `features/auth/repository/session.repository.ts` → `createSession(userId, refreshTokenHash, ip, userAgent)`,
  chamado pelos dois caminhos que criam sessão: login por senha e callback do Google.
- Dentro da mesma transação, o método apaga todas as sessões daquele usuário exceto as 5 de `createdAt`
  mais recente, e **devolve os `sessionId` apagados**. O service grava cada um na denylist (regra 2).
- O sexto login derruba a sessão mais antiga na hora: o refresh dela não funciona mais, e o access
  token dela cai no `401` da denylist na requisição seguinte.
- Dois logins simultâneos do mesmo usuário podem tentar apagar as mesmas linhas; apagar uma linha já
  apagada não é erro, e o resultado final continua sendo as 5 mais recentes.

### 17. Um job diário apaga o que venceu

Nada nesta API remove uma `session` ou uma `verification` vencida sozinho. Linha que ninguém lê fica
para sempre.

- `core/queue/` registra `@nestjs/bullmq` contra o `REDIS_URL`, sobre o mesmo cliente `ioredis` da
  regra 11.
- `src/worker.ts` é a entrada do processo separado, com o próprio container: cria a fila
  `auth-maintenance` e registra o job repetível `purge-expired-auth-records` com
  `repeat: { pattern: "0 3 * * *", tz: "America/Sao_Paulo" }`.
- **O agendamento é do BullMQ, não do `@nestjs/schedule`.** Um `@Cron` roda em toda instância do
  processo; um job repetível do BullMQ é uma chave no Redis, então duas instâncias de worker produzem
  uma execução, não duas.
- `features/auth/job/purge-expired-auth-records.job.ts` é o `@Processor` e chama o service, como o
  controller faz.
- O job apaga:

  | Tabela | Condição |
  | --- | --- |
  | `session` | `expiresAt` anterior a agora |
  | `verification` | `expiresAt` anterior a agora |
  | `emailDispatch` | `createdAt` anterior a agora menos 24 horas |

- **Sessão vencida não vai para a denylist.** O access token dela expirou por conta própria muito antes,
  e o refresh é conferido contra a tabela.
- **O job é idempotente**: rodar duas vezes seguidas apaga na segunda zero linhas.
- Cada execução registra no Pino a quantidade apagada por tabela.
- Em desenvolvimento o worker roda por `npm run worker`. O container do worker na stack inteira é a
  issue #4.

---

## Erros

Catálogo próprio desta API. No corpo, o código sai como `type`, na forma
`tag:clinicore.com.br,2026:<código em minúsculas e com hífen>`, e a mensagem sai como `title` (regra 10).
A mensagem é inglês, texto de desenvolvedor; o texto que a pessoa lê é escrito no web a partir do `type`.

| Código | HTTP | Quando | Mensagem |
| --- | --- | --- | --- |
| `VALIDATION_FAILED` | `400` | Corpo ou query fora do DTO; `errors` traz o campo e a restrição | "Validation failed" |
| `INVALID_TOKEN` | `400` | Token de reset inválido, consumido ou expirado em `POST /password-resets/confirmation`; token de verificação inválido, consumido ou inexistente em `POST /email-verifications/confirmation` | "Invalid token" |
| `TOKEN_EXPIRED` | `400` | Token de verificação expirado em `POST /email-verifications/confirmation` | "Token expired" |
| `INVALID_PASSWORD` | `400` | `currentPassword` incorreta em `change-password` | "Invalid password" |
| `INVALID_CREDENTIALS` | `401` | E-mail inexistente, ou senha incorreta no login | "Invalid email or password" |
| `INVALID_SESSION` | `401` | Cookie de acesso ausente, malformado, expirado ou na denylist; refresh de sessão inexistente ou vencida | "Invalid session" |
| `SESSION_REUSED` | `401` | Refresh token que não bate com o gravado; a sessão é derrubada (regra 3) | "Refresh token reuse detected" |
| `EMAIL_NOT_VERIFIED` | `403` | Login com a senha correta e o e-mail ainda não verificado | "Email not verified" |
| `INVALID_ORIGIN` | `403` | Método que não é `GET`, `HEAD` nem `OPTIONS` com `Origin` fora de `ALLOWED_ORIGINS`, ou sem o header | "Invalid origin" |
| `RATE_LIMITED` | `429` | Limite da rota excedido (regra 11) | "Too many requests" |
| `SERVICE_UNAVAILABLE` | `503` | Redis inalcançável numa requisição autenticada (regra 2) ou numa rota com limite (regra 11) | "Service temporarily unavailable" |

Fora do catálogo, todo erro sai como `about:blank` com a frase padrão do status no `title`: o
`HttpException` levantado pelo próprio framework (`404`, `405`, `413`) e todo `500` (regra 10).

Códigos que saem apenas em `?error=` de um `302`, sem corpo e sem `message`:

| Código | Rota | Quando |
| --- | --- | --- |
| `INVALID_STATE` | `/oauth/google/callback` | `state` da query diferente do cookie, ou cookie ausente |
| `UNVERIFIED_PROVIDER_EMAIL` | `/oauth/google/callback` | O Google devolveu `email_verified` falso |
| `PROVIDER_ERROR` | `/oauth/google/callback` | A troca do `code` falhou, ou o Google devolveu erro |

## Efeitos Colaterais

- **Persistência:** cadastro de e-mail novo grava uma linha em `user`, uma em `account`
  (`provider = "credential"`), uma em `emailDispatch` e uma em `verification`. Cadastro de e-mail
  existente não grava nada. Login grava uma linha em `session` e apaga as excedentes da regra 16.
  Refresh atualiza `refreshTokenHash` e `expiresAt` da `session`; refresh reusado apaga a linha.
  Logout apaga a linha de `session`. Verificação de e-mail marca `user.emailVerified = true`, consome
  os tokens pendentes daquele endereço e cria uma `session`. Pedido de reset aceito grava uma linha em
  `emailDispatch` e uma em `verification`. Reset de senha consome a linha de `verification`, atualiza
  ou cria `account.passwordHash` e apaga todas as sessões do usuário. O job da regra 17 apaga linhas
  vencidas.
- **Redis:** cada requisição limitada incrementa um contador com TTL (regra 11); cada sessão revogada
  grava `auth:revoked:<sessionId>` com TTL de 900 segundos (regra 2); o job repetível da regra 17 é uma
  chave da fila `auth-maintenance`.
- **Concorrência:**
  - dois cadastros simultâneos com o mesmo e-mail — o índice único de `user.email` reprova o segundo, o
    repository traduz para `Conflict` e o service responde `202` igual (regra 8);
  - dois usos simultâneos do mesmo refresh token — a transação serializa, o segundo cai na detecção de
    reuso e a sessão morre (regra 3);
  - dois usos simultâneos do mesmo token de reset ou de verificação — o token é consumido dentro da
    transação, e o segundo responde `400 INVALID_TOKEN`;
  - dois pedidos simultâneos de e-mail para o mesmo endereço dentro da janela — a checagem e a gravação
    em `emailDispatch` acontecem numa transação só, e o segundo é barrado;
  - dois logins simultâneos do mesmo usuário — regra 16.
- **Transação:** cadastro por senha (`user`, `account`, `emailDispatch`, `verification`), cadastro por
  Google (`user`, `account`), rotação do refresh, criação de sessão com o corte do teto, consumo de
  token com troca de senha, consumo de token com verificação de e-mail, e a checagem-e-gravação de
  `emailDispatch`. **O envio de e-mail nunca está dentro de uma transação** (regra 13).

---

## Cenários de Aceite (Gherkin)

Todos os cenários sobem o módulo com `Test.createTestingModule` e batem na rota com `supertest`,
contra o **Postgres real e o Redis real** do `compose.yaml` da API, no padrão do contrato do repo. Não
se faz mock de repository nem de `DataSource`. O e2e que sobe o `AppModule` inteiro fica em `test/`.

Duas fronteiras externas são substituídas, e nenhuma outra:

- **o transport do Nodemailer**, por um duplo que enfileira as mensagens em memória, para que o teste
  conte os envios e leia o token do link;
- **os endpoints do Google** — `https://oauth2.googleapis.com/token` e
  `https://www.googleapis.com/oauth2/v3/userinfo` —, por `nock`, porque o `passport-oauth2` faz essa
  troca pelo módulo `https` do Node e não pelo `fetch` global.

### Cenário 1 — Cadastro cria o usuário e envia o link (caminho feliz, regras 5 e 12)

```gherkin
Dado que o e-mail `ana@exemplo.com` não existe na tabela `user`
Quando é enviado `POST /users` com nome, e-mail e a senha `Clinica#2026`
Então o sistema responde `202` com corpo vazio
E existe uma linha em `user` com `email = "ana@exemplo.com"` e `emailVerified = false`
E existe uma linha em `account` com `provider = "credential"` e `passwordHash` preenchida
E `user.id` e `account.userId` são o mesmo UUID, gerado pelo banco
E existe uma linha em `verification` com `purpose = "email_verification"` e `consumedAt` nulo
E um e-mail com o assunto `Confirme seu e-mail no Clinicore` foi entregue ao transport
E existe exatamente uma linha em `emailDispatch` para esse endereço, com `kind = "verification"`
E nenhum `Set-Cookie` é devolvido
```

### Cenário 2 — Cadastro repetido não grava, não envia e não revela a conta (exceção, regra 8)

```gherkin
Dado um usuário cadastrado com `email = "ana@exemplo.com"`
Quando é enviado `POST /users` com o mesmo e-mail, repetido 3 vezes
Então as 3 respostas são `202` com corpo vazio, idênticas à do Cenário 1
E continua existindo exatamente uma linha em `user` e uma em `account` para esse e-mail
E nenhum e-mail foi entregue ao transport
E dois `POST /users` disparados ao mesmo tempo com o mesmo e-mail novo respondem os dois `202`
E resta exatamente uma linha em `user` para esse e-mail
```

### Cenário 3 — Login antes de verificar é bloqueado e reenvia o link (exceção, regra 5)

```gherkin
Dado um usuário cadastrado com `emailVerified = false` e sem pedido de e-mail nos últimos 60 segundos
Quando é enviado `POST /sessions` com a senha correta
Então o sistema responde `403` com o código `EMAIL_NOT_VERIFIED`
E nenhuma linha é criada em `session`
E um e-mail de confirmação foi entregue ao transport
E com a senha errada a resposta é `401 INVALID_CREDENTIALS` e nenhum e-mail é entregue
```

### Cenário 4 — Verificar o e-mail não abre sessão, e repetir é recusado (caminho feliz, regra 5)

```gherkin
Dado um usuário cadastrado com `emailVerified = false` e dois links de verificação válidos
Quando é feita a requisição `POST /email-verifications/confirmation` com o token do primeiro link
Então o sistema responde `204` sem `Set-Cookie`
E `user.emailVerified` passa a ser `true`
E nenhuma linha em `session` é criada
E as duas linhas em `verification` desse endereço têm `consumedAt` preenchido
E repetir a requisição com qualquer um dos dois tokens responde `400` com o código `INVALID_TOKEN`
```

### Cenário 5 — Token de verificação inválido ou expirado é recusado com o código (exceção, regra 5)

```gherkin
Dado um usuário cadastrado com `emailVerified = false`
Quando `POST /email-verifications/confirmation` é chamado com um token que não existe
Então o sistema responde `400` com o código `INVALID_TOKEN`
E com um token de `expiresAt` no passado responde `400` com o código `TOKEN_EXPIRED`
E nenhuma das duas respostas traz `Set-Cookie`
E nenhuma linha em `user` ou em `session` é alterada
```

### Cenário 6 — Login abre a sessão com os dois cookies (caminho feliz, regras 2 e 3)

```gherkin
Dado um usuário com `emailVerified = true`
Quando é enviado `POST /sessions` com a senha correta
Então o sistema responde `201` com `Location: /sessions/current` e o corpo `{ "user": { ... } }`, sem senha nem hash
E o `Set-Cookie` de `clinicore_access` traz `HttpOnly`, `Path=/` e `Max-Age=900`
E o `Set-Cookie` de `clinicore_refresh` traz `HttpOnly`, `Path=/sessions/current/tokens` e `Max-Age=86400`
E a linha criada em `session` tem `expiresAt` 24 horas à frente de `createdAt`, com tolerância de 60 segundos
E `session.refreshTokenHash` não contém o valor que veio no cookie
E `GET /sessions/current` com o cookie de acesso devolve `200` com o mesmo usuário
```

### Cenário 7 — Senha errada não distingue de e-mail inexistente (exceção, regra 8)

```gherkin
Dado um usuário verificado com o e-mail `ana@exemplo.com`
Quando é enviado `POST /sessions` com a senha errada
Então o sistema responde `401` com o código `INVALID_CREDENTIALS`
E a mesma requisição para o e-mail inexistente `ninguem@exemplo.com` responde `401` com o mesmo código
E os dois corpos de resposta são iguais
E a diferença entre as medianas de 20 respostas de cada caso fica abaixo de 50 milissegundos
```

### Cenário 8 — O refresh rotaciona e devolve cookies novos (caminho feliz, regra 3)

```gherkin
Dado um usuário logado, com o cookie `clinicore_refresh` da resposta do login
Quando é enviado `POST /sessions/current/tokens` com esse cookie
Então o sistema responde `204`
E os dois `Set-Cookie` trazem valores diferentes dos anteriores
E `session.refreshTokenHash` mudou e `session.id` continua o mesmo
E `session.expiresAt` foi empurrado para 24 horas à frente
E o cookie de acesso novo autentica `GET /sessions/current`
```

### Cenário 9 — Reusar o refresh antigo derruba a sessão inteira (exceção, regra 3)

```gherkin
Dado um usuário logado e um `POST /sessions/current/tokens` já executado com sucesso
Quando é enviado `POST /sessions/current/tokens` de novo com o cookie de refresh **antigo**
Então o sistema responde `401` com o código `SESSION_REUSED`
E a linha em `session` deixa de existir
E a chave `auth:revoked:<sessionId>` existe no Redis com TTL menor ou igual a 900
E o cookie de acesso emitido na rotação, ainda dentro dos 15 minutos, passa a responder `401 INVALID_SESSION` em `GET /sessions/current`
E o cookie de refresh da rotação também responde `401 INVALID_SESSION`
```

### Cenário 10 — Logout derruba a sessão na hora, mesmo com o access token vivo (caminho feliz, regra 2)

```gherkin
Dado um usuário logado com uma linha em `session`
Quando é enviado `DELETE /sessions/current` com o cookie de acesso
Então o sistema responde `204`
E os dois `Set-Cookie` de expiração são devolvidos, com `Max-Age=0`
E a linha em `session` deixa de existir
E a chave `auth:revoked:<sessionId>` existe no Redis
E `GET /sessions/current` com o mesmo cookie de acesso responde `401` com o código `INVALID_SESSION`
E repetir `DELETE /sessions/current` com o mesmo cookie responde `401` e não altera nenhuma tabela
```

### Cenário 11 — Senha fraca é recusada nas três rotas (exceção, regra 4)

```gherkin
Dado o decorator de política de senha ativo nos DTOs
Quando `sem_maiuscula#1`, `SEM_DIGITO#a`, `SemEspecial1` ou `Aa#1` são enviados como senha
Então cada um responde `400` com o código `VALIDATION_FAILED`
E `errors` traz o `pointer` do campo da senha com o `code` `weak_password`
E o mesmo vale nas rotas `/users`, `/password-resets/confirmation` e `/users/me/password`
E nenhuma linha é gravada em `user`, `account` ou `verification`
E `Clinica#2026` é aceita nas três
```

### Cenário 12 — Campo desconhecido no corpo é recusado (exceção, regras 1 e 10)

```gherkin
Dado o `ValidationPipe` global com `forbidNonWhitelisted`
Quando é enviado `POST /users` com um campo `callbackURL` igual a `http://evil.example/x`
Então o sistema responde `400` com o código `VALIDATION_FAILED`
E nenhuma linha é gravada em `user`
E o mesmo vale para `redirectTo` em `POST /password-resets`
```

### Cenário 13 — Google vincula à conta existente em vez de duplicar (caminho alternativo, regra 6)

```gherkin
Dado um usuário verificado com `email = "ana@exemplo.com"` e uma linha em `account` com `provider = "credential"`
E os endpoints do Google substituídos, devolvendo `sub = "google-ana"`, `email = "ana@exemplo.com"` e `email_verified = true`
Quando é feita a requisição `GET /oauth/google` e guardado o cookie `clinicore_oauth_state`
E é feita a requisição `GET /oauth/google/callback` com o `state` da URL de autorização, um `code` qualquer e esse cookie
Então o sistema responde `302` para `http://localhost:3000/app`, com os dois cookies de sessão
E continua existindo exatamente uma linha em `user` com esse e-mail
E passa a existir uma segunda linha em `account` com `provider = "google"`, `providerAccountId = "google-ana"` e o mesmo `userId`
E nenhuma coluna de `account` guarda token do Google
E entrar com a senha original continua funcionando
```

### Cenário 14 — Iniciar o login com Google não grava no banco (caminho alternativo, regra 6)

```gherkin
Dado a contagem de linhas de todas as tabelas do schema público
Quando é feita a requisição `GET /oauth/google` 3 vezes
Então cada resposta é `302` para `accounts.google.com`, com `prompt=select_account`
E cada resposta traz o cookie `clinicore_oauth_state` com `HttpOnly`, `Path=/oauth/google` e `Max-Age=600`
E nenhuma tabela ganhou linha
```

### Cenário 15 — `state` adulterado e e-mail não verificado pelo Google são recusados (exceção, regra 6)

```gherkin
Dado os endpoints do Google substituídos
Quando `GET /oauth/google/callback` é chamado com um `state` diferente do cookie
Então o sistema responde `302` para `http://localhost:3000/login?error=INVALID_STATE`
E chamado sem o cookie `clinicore_oauth_state` responde o mesmo
E com o `state` correto mas `email_verified` falso responde `302` para `http://localhost:3000/login?error=UNVERIFIED_PROVIDER_EMAIL`
E nenhuma linha é gravada em `user`, `account` ou `session`
```

### Cenário 16 — Recuperação de senha não revela quem tem conta (exceção, regras 7 e 8)

```gherkin
Dado um usuário verificado com o e-mail `ana@exemplo.com` e sem pedidos de e-mail nas últimas 24 horas
Quando é enviado `POST /password-resets` para `ana@exemplo.com` e depois para `ninguem@exemplo.com`
Então as duas requisições respondem `202` com corpo vazio
E um e-mail com o assunto `Redefinir sua senha do Clinicore` foi entregue apenas para `ana@exemplo.com`
E o link do e-mail aponta para `http://localhost:3000/reset-password?token=<token>`
E existe exatamente uma linha em `verification` com `purpose = "password_reset"`
```

### Cenário 17 — Pedidos repetidos para o mesmo endereço não gravam nem enviam (exceção, regra 15)

```gherkin
Dado um usuário verificado com o e-mail `ana@exemplo.com` e sem pedidos de e-mail nas últimas 24 horas
Quando `POST /password-resets` para `ana@exemplo.com` é enviado 4 vezes em 10 segundos, cada vez de um IP de cliente diferente
Então as 4 respostas são `202` com corpo vazio
E exatamente 1 e-mail foi entregue ao transport
E existe exatamente 1 linha em `verification` com `purpose = "password_reset"`
E existe exatamente 1 linha em `emailDispatch` para esse endereço com `kind = "password_reset"`
```

### Cenário 18 — O teto diário por endereço segura o sexto pedido (exceção, regra 15)

```gherkin
Dado o e-mail `ana@exemplo.com` com 5 linhas em `emailDispatch` de `kind = "verification"` nas últimas 24 horas, a mais recente há 2 minutos
Quando é enviado `POST /email-verifications` para esse endereço
Então o sistema responde `202` com o mesmo corpo vazio de um pedido aceito
E nenhum e-mail é entregue ao transport
E continua havendo 5 linhas em `emailDispatch` para esse endereço
E o mesmo pedido de `kind = "password_reset"` para o mesmo endereço continua sendo aceito
```

### Cenário 19 — O limite por endereço vale também para e-mail sem cadastro (exceção, regra 15)

```gherkin
Dado o e-mail `ninguem@exemplo.com`, que não existe em `user`
Quando `POST /password-resets` para esse endereço é enviado 2 vezes em 10 segundos
Então existe exatamente 1 linha em `emailDispatch` para esse endereço
E as 2 respostas são iguais às de um endereço cadastrado no Cenário 17
```

### Cenário 20 — Redefinir a senha derruba todas as sessões na hora (caminho feliz, regras 2 e 7)

```gherkin
Dado um usuário verificado logado em dois dispositivos, com duas linhas em `session`, e um token de reset válido
Quando é enviado `POST /password-resets/confirmation` com a nova senha `Outra#Senha9`
Então o sistema responde `204`
E nenhuma linha em `session` resta para esse usuário
E existe uma chave `auth:revoked:<sessionId>` no Redis para cada uma das duas sessões
E `GET /sessions/current` com qualquer um dos dois cookies de acesso responde `401 INVALID_SESSION`
E entrar com `Outra#Senha9` funciona
E entrar com a senha antiga responde `401`
E repetir a mesma requisição responde `400` com o código `INVALID_TOKEN`
```

### Cenário 21 — Quem entrou só com Google ganha login por senha pelo reset (caminho alternativo, regra 7)

```gherkin
Dado uma linha em `user` com `email = "ana@exemplo.com"` e `emailVerified = true`
E uma única linha em `account` para ela, com `provider = "google"`
Quando é pedido o reset de senha e o token recebido é usado com a senha `Clinica#2026`
Então passa a existir uma linha em `account` com `provider = "credential"` e `passwordHash` preenchida
E a linha com `provider = "google"` continua existindo
E entrar com `ana@exemplo.com` e `Clinica#2026` responde `201`
```

### Cenário 22 — Trocar a senha mantém a sessão atual e derruba as outras (caminho feliz, regra 7)

```gherkin
Dado um usuário logado em dois dispositivos, com duas linhas em `session`
Quando é enviado `PUT /users/me/password` com a senha atual correta e a nova senha, usando o cookie do primeiro dispositivo
Então o sistema responde `204`
E resta exatamente uma linha em `session`, a do cookie usado na requisição
E `GET /sessions/current` com o cookie do primeiro dispositivo continua respondendo `200`
E `GET /sessions/current` com o cookie do segundo responde `401 INVALID_SESSION`
E repetir a mesma requisição responde `400` com o código `INVALID_PASSWORD`
E a senha continua a da primeira chamada
```

### Cenário 23 — O sexto login derruba a sessão mais antiga (exceção, regra 16)

```gherkin
Dado um usuário verificado com 5 linhas em `session`
Quando é enviado `POST /sessions` com a senha correta
Então o sistema responde `201`
E o usuário continua com exatamente 5 linhas em `session`
E a linha de `createdAt` mais antiga deixou de existir
E `GET /sessions/current` com o cookie de acesso dessa sessão responde `401 INVALID_SESSION`
E 20 logins seguidos, respeitando a regra 11, deixam o usuário com exatamente 5 linhas em `session`
```

### Cenário 24 — CORS libera só quem está na lista, e a guarda de `Origin` barra o resto (exceção, regra 1)

```gherkin
Dado a API rodando com `ALLOWED_ORIGINS` igual a `https://app.clinicore.com.br,https://clinicore.com.br`
Quando chega um preflight `OPTIONS /sessions` com `Origin: https://app.clinicore.com.br`
Então a resposta traz `Access-Control-Allow-Origin: https://app.clinicore.com.br`
E traz `Access-Control-Allow-Credentials: true`
E nunca traz `Access-Control-Allow-Origin: *`
E o mesmo preflight com `Origin: https://clinicore.com.br` é liberado com essa origem
E o preflight de `DELETE /sessions/current` traz `DELETE` em `Access-Control-Allow-Methods`
E `POST /sessions/current/tokens` com `Origin: http://evil.example` responde `403 INVALID_ORIGIN`
E `POST /sessions/current/tokens` com `Origin: https://clinicore.com.br.evil.example` responde `403 INVALID_ORIGIN`
E `POST /sessions/current/tokens` sem o header `Origin` responde `403 INVALID_ORIGIN`
E `DELETE /sessions/current` sem o header `Origin` responde `403 INVALID_ORIGIN`
E `GET /sessions/current` sem o header `Origin` responde normalmente
```

### Cenário 25 — O limite do login barra a sexta tentativa (exceção, regra 11)

```gherkin
Dado a API com o Redis limpo
Quando seis requisições `POST /sessions` com senha errada chegam do mesmo IP dentro de 60 segundos
Então as cinco primeiras respondem `401`
E a sexta responde `429` com o código `RATE_LIMITED`
E `GET /health` chamado 200 vezes seguidas nunca responde `429`
```

### Cenário 26 — Forjar `x-forwarded-for` não escapa do limite (exceção, regra 11)

```gherkin
Dado a API com `TRUSTED_PROXIES` igual a `10.0.0.0/8` e o Redis limpo
Quando seis requisições `POST /sessions` com senha errada são enviadas dentro de 60 segundos
E cada uma traz `x-forwarded-for: <forjado>, 203.0.113.7, 10.0.0.1`, com um `<forjado>` diferente a cada chamada
Então as cinco primeiras respondem `401`
E a sexta responde `429`
E existe uma única chave de contagem no Redis para `/sessions`, a de `203.0.113.7`
```

No teste automatizado, `TRUSTED_PROXIES` vale `127.0.0.1/32,10.0.0.0/8`: o socket do supertest é
`127.0.0.1` e faz o papel do proxy. Só com `10.0.0.0/8`, o Express não confiaria no socket, ignoraria o
`x-forwarded-for` inteiro, e o cenário passaria sem provar nada.

### Cenário 27 — O log registra a requisição sem vazar segredo (caminho feliz, regra 9)

```gherkin
Dado a API com `LOG_LEVEL` igual a `info`
Quando é enviado `POST /sessions` com senha correta
Então a saída em `stdout` contém uma linha JSON com o método, o caminho, o status `200` e a duração
E essa linha não contém a senha enviada
E não contém o valor do header `cookie` nem do `set-cookie`
E uma requisição `GET /health` não produz nenhuma linha de log
```

### Cenário 28 — Erro desconhecido vira 500 sem detalhe (exceção, regra 10)

```gherkin
Dado uma rota de teste que lança um erro não tratado
Quando ela é chamada
Então o sistema responde `500`
E o corpo é exatamente `{"type":"about:blank","title":"Internal Server Error","status":500}`
E o `Content-Type` é `application/problem+json`
E a stack completa aparece no log do Pino
E a stack não aparece na resposta
```

### Cenário 29 — Ambiente incompleto derruba o boot listando o que falta (exceção, regra 14)

```gherkin
Dado o ambiente sem `JWT_SECRET` e sem `TRUSTED_PROXIES`
Quando a API é inicializada
Então o processo escreve em `stderr` a linha `Invalid environment:`
E escreve a linha `  JWT_SECRET: expected a string with at least 32 characters`
E escreve a linha `  TRUSTED_PROXIES: expected a comma-separated list of CIDR blocks (10.0.0.0/8,…)`
E sai com código `1`
E não abre a porta HTTP
E com `MAIL_FROM` diferente de `SMTP_USER` escreve `  MAIL_FROM: expected an email address equal to SMTP_USER`
E o worker, inicializado com o mesmo ambiente, falha da mesma forma
```

### Cenário 30 — Os cookies são `HttpOnly`, `SameSite=Lax` e `Secure` em produção (caminho alternativo, regra 2)

```gherkin
Dado a API inicializada com `NODE_ENV` igual a `production`
Quando um login bem-sucedido devolve os cookies de sessão
Então os dois `Set-Cookie` trazem `Secure`, `HttpOnly` e `SameSite=Lax`
E nenhum dos dois traz `SameSite=None`
E nenhum dos dois traz `Domain`
E com `NODE_ENV` igual a `development` o mesmo login devolve `HttpOnly` e `SameSite=Lax`, sem `Secure`
```

### Cenário 31 — Redis fora do ar fecha a porta em vez de abrir (exceção, regra 2)

```gherkin
Dado um usuário logado e o Redis inalcançável
Quando é feita a requisição `GET /sessions/current` com o cookie de acesso válido
Então o sistema responde `503` com o código `SERVICE_UNAVAILABLE`
E `DELETE /sessions/current` responde `503` com o mesmo código e a linha em `session` continua existindo
E `GET /health` continua respondendo `200`
```

### Cenário 32 — SMTP fora do ar não desfaz o cadastro nem muda a resposta (exceção, regra 13)

```gherkin
Dado que o e-mail `ana@exemplo.com` não existe na tabela `user`
E o transport de e-mail falha ao enviar
Quando é enviado `POST /users` com nome, e-mail e a senha `Clinica#2026`
Então o sistema responde `202` com corpo vazio
E existe uma linha em `user` e uma em `account` para esse e-mail
E existe uma linha em `emailDispatch` para esse endereço
E o log do Pino tem uma linha de nível `error` com o endereço e o motivo da falha
```

### Cenário 33 — O job apaga o que venceu e nada mais, e repetir não apaga de novo (caminho feliz, regra 17)

```gherkin
Dado uma `session` vencida e uma válida
E uma `verification` vencida e uma válida
E uma linha de `emailDispatch` de 25 horas atrás e uma de 1 hora atrás
Quando o job `purge-expired-auth-records` é executado
Então só as três linhas vencidas deixam de existir
E o log registra 1 linha apagada por tabela
E nenhuma chave `auth:revoked:` foi criada no Redis
E executar o job de novo apaga 0 linhas
```

### Cenário 34 — O worker agenda o job uma única vez por dia (caminho feliz, regra 17)

```gherkin
Dado o worker iniciado contra o Redis de teste
Quando o worker é iniciado uma segunda vez com a mesma configuração
Então existe exatamente um job repetível `purge-expired-auth-records` na fila `auth-maintenance`
E o padrão do agendamento é `0 3 * * *` no fuso `America/Sao_Paulo`
```

### Cenário 35 — A entity e a migration não divergem (caminho feliz, regra 12)

```gherkin
Dado o banco com todas as migrations aplicadas
Quando `typeorm migration:generate` é executado apontando para um caminho temporário
Então esse caminho continua não existindo
E o step do CI falha se ele passar a existir
```

---

## Fora de Escopo

- **As telas do `apps/web`** — elas estão na spec irmã `003-autenticacao-web.md`, já reescrita contra
  as rotas desta spec pela ADR 0002. As duas descrevem a mesma entrega por perfis diferentes: aqui o
  contrato HTTP, lá a tela. Em divergência entre as duas, **esta é a autoridade**.
- **Organização, clínica, rede, papéis e permissões** — issues #6 e #7.
- **Convite de usuário** — decidido que não existe: o cadastro é público. A issue #8 precisa ser
  reescrita como gestão de usuários já cadastrados.
- **E-mail pela fila** — a fila desta entrega serve só ao job de limpeza (regra 13).
- **Remetente próprio no e-mail transacional** — fica no Gmail da conta de `SMTP_USER` até o
  lançamento público (regra 13).
- **Trilha de auditoria de acesso ao prontuário** — issue #9.
- **2FA, sessões ativas por dispositivo, revogar sessão específica, excluir conta, trocar e-mail e
  editar perfil** — não pedidos.
- **Outros provedores sociais além do Google** — não pedidos.
- **Teto absoluto de duração de sessão**, além das 24 horas de inatividade — decidido deixar de fora.
- **Limite de requisições em `/health`** — decidido deixar de fora (regra 11).
- **Dockerfile, `compose.yaml` da raiz, container do worker e deploy de homolog** — issue #4, que
  também define o valor de `TRUSTED_PROXIES` em homolog e garante que a API só é alcançável pelo proxy
  que acrescenta o hop em `x-forwarded-for` (regra 11).

## Onde moram as strategies e os guards

A `JwtStrategy` e a `GoogleStrategy` do `@nestjs/passport` são providers da feature `auth` e ficam em
`features/auth/strategy/`: elas declaram como um credencial vira usuário, e isso é regra desta
feature. Os guards que delas dependem — `JwtAuthGuard` e `OriginGuard` — ficam em `common/guards/`,
porque são registrados como `APP_GUARD` e valem para toda feature futura, junto dos decorators
`@Public()` e `@CurrentUser()` em `common/decorators/`.

Essa divisão está no contrato do repo, na seção **`api` — camadas**.

## Quebra em Tasks

| # | Issue | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- | --- |
| 1 | #65 | Add the authentication environment, the logger and the error contract to apps/api | As 12 variáveis em `core/config/`, com o decorator de CIDR e o de `MAIL_FROM`; `core/logger/` com o Pino atrás do `LoggerService` e o interceptor; `common/exceptions/business-error.ts`; `common/filters/business-error.filter.ts`; o `ValidationPipe` global e o `exceptionFactory` de `fields`; `common/guards/origin.guard.ts`; `app.set("trust proxy", …)`; `.github/workflows/ci.yml` com as variáveis novas e o serviço `redis:8` | Cenários 27, 28 e 29 verdes, sem a linha do worker no 29; a parte de `Origin` do Cenário 24 verde; os quatro gates da API saem com código 0 | — |
| 2 | #69 | Rate-limit the authentication routes by trusted client IP | `@nestjs/throttler` com `@nest-lab/throttler-storage-redis` sobre o cliente `ioredis` de `core/redis/`, o guard global com `getTracker` sobre `req.ip` normalizado e falha fechada em `503`, o `@SkipThrottle()` de `/health` e a tabela de limites da regra 11 | Cenários 25 e 26 verdes | #65 |
| 3 | #66 | Authenticate with email and password and issue the session cookies | Entities e a migration `AddAuth`; `@node-rs/argon2`; `sign-up`, `sign-in`, `refresh`, `sign-out` e `session`; `features/auth/strategy/jwt.strategy.ts` e `common/guards/jwt-auth.guard.ts` com `@Public()`; a denylist no Redis; a rotação com detecção de reuso; o teto de 5 sessões; o step de migration no CI | Cenários 2, 6, 7, 8, 9, 10, 11, 12, 23, 24, 30, 31 e 35 verdes | #69 |
| 4 | #67 | Send the verification and the reset emails with a per-address limit | `core/mail/` com o transport do Gmail e os dois templates; a migration `AddEmailDispatch`; `email-dispatch.repository.ts` com `registerDispatch`; o fluxo de verificação de e-mail e o de reset de senha; o reenvio no login não verificado | Cenários 1, 3, 4, 5, 16, 17, 18, 19, 20, 21 e 32 verdes | #66 |
| 5 | #68 | Sign in with Google and link it to the existing account | `features/auth/strategy/google.strategy.ts` com `passport-google-oauth20` e o `store` de `state` em cookie; as duas rotas; a vinculação à conta existente; a recusa de `email_verified` falso; o `nock` dos endpoints do Google nos testes | Cenários 13, 14 e 15 verdes | #66 |
| 6 | #70 | Change the password of the signed-in user | `PUT /users/me/password`, a conferência de `currentPassword` com argon2 e a derrubada das outras sessões com denylist | Cenário 22 verde | #66 |
| 7 | #71 | Purge expired authentication records daily in a worker | `core/queue/` com `@nestjs/bullmq` sobre o cliente de #69; `src/worker.ts` e o script `worker`; o job repetível `purge-expired-auth-records`; o `@Processor`, o service e os repositories da limpeza | Cenários 33 e 34 verdes; a linha do worker no Cenário 29 verde | #67, #69 |

As tasks do `apps/web` ficam na spec irmã, reescrita pela ADR 0002 — o web permanece em Vite e TanStack Router, e passa a falar com estas rotas por axios.

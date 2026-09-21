# 004 — Entrar, cadastrar-se e usar passkey no app e no web (usuário da clínica)

> **Status:** publicada
> **Perfil:** UI
> **Módulo:** `apps/mobile` (novo) e `apps/web`
> **Epic:** #91, dentro da #1 — Plataforma; as tasks de passkey em #92, dentro da #90 — Beta com a clínica piloto
> **Spec irmã:** `docs/specs/autenticacao/004-sessao-por-token-api.md` (perfil API)
> **Base:** `docs/specs/autenticacao/003-autenticacao-web.md`. As telas do app são as do web, com as
> diferenças desta spec; em todo o resto, as regras da 003-web valem para o app.
> **Decisões:** `docs/decisions/0007-app-nativo-em-flutter-com-offline.md`

## Acceptance Criteria

### Referência visual

`N/A` — sem protótipo. O app repete a estrutura das telas do web (regra 6 da 003-web: cartão com
16px de margem, no máximo 400px de largura). Tema, cores e tipografia continuam fora, como no web.

### Especificação das telas

**No `apps/mobile`**, sete telas. Os caminhos são os do `go_router` e repetem os do web.

| Caminho | Tela | Acesso |
| --- | --- | --- |
| `/login` | Entrar | público |
| `/signup` | Criar conta | público |
| `/verify-email` | Confirme seu e-mail | público |
| `/forgot-password` | Recuperar senha | público |
| `/app` | Início | sessão válida |
| `/app/account/password` | Trocar senha | sessão válida |
| `/app/account/passkeys` | Biometria e passkeys | sessão válida, só no Android (regra 7) |

- **`/login`, `/signup` e `/forgot-password`** — os mesmos campos, botões e links do web.
- **`/verify-email`** — sem o estado de `token`: o link do e-mail abre no web (regra 9). Mostra a
  instrução com o e-mail do cadastro, o botão `Reenviar link` e o link `Já confirmei, entrar`.
- **`/app`** — nome e e-mail do usuário, os links `Trocar senha` e, no Android,
  `Biometria e passkeys`, e o botão `Sair`.
- **`/app/account/password`** — igual ao web.
- **`/app/account/passkeys`** — a lista de passkeys do usuário, cada uma com a data de criação e a de
  último uso e o botão `Remover`; o botão `Adicionar passkey`; o link `Voltar`.
- **Não existem no app** `/reset-password` nem o estado de `token` de `/verify-email`: os dois vivem
  no web (regra 9).

**No `apps/web`**, uma tela nova e duas mudanças:

| Rota | Mudança |
| --- | --- |
| `/login` | O campo E-mail oferece as passkeys salvas no autocompletar (regra 6) |
| `/app` | Oferta de ativar a biometria depois do login (regra 7); link `Biometria e passkeys` |
| `/app/account/passkeys` | Nova — a mesma tela do app |

### Caminho de menu

`N/A` — não existe menu, como na 003-web. `Biometria e passkeys` é alcançada pelo link em `/app`.

### Perfis e privilégios

| Perfil / Ação | Privilégio | Observação |
| --- | --- | --- |
| Visitante sem sessão | — | Vê `/login`, `/signup`, `/verify-email` e `/forgot-password` |
| Usuário autenticado | sessão válida | Vê `/app`, `/app/account/password` e `/app/account/passkeys` |
| Usuário autenticado em tela pública | sessão válida | É levado para `/app` |

**Não existe papel nesta entrega.**

---

## Regras de Negócio

### 1. Restrição de Acesso

- No app, o `redirect` do `go_router` é o único guardião de `/app`: sem sessão, leva para `/login`
  guardando o destino em `redirect`; com sessão, `/login` e `/signup` levam para `/app`. É a regra 1 da
  003-web, no app.
- **Ao abrir o app**, a sessão é lida do armazenamento seguro (regra 3) e confirmada com
  `GET /sessions/current`. Sem token guardado, a primeira tela é `/login`.

#### Origem do fluxo

O usuário chega abrindo o app, pelo redirecionamento da guarda, ou pelos links entre as telas.

### 2. O app fala com a API por um cliente gerado do OpenAPI

- O cliente HTTP é gerado com `swagger_parser` 1.44.3 a partir de `/api-json` da API local, em Retrofit
  sobre `dio` 5.11.1, e **o código gerado é commitado**. Regenerar é um comando do `apps/mobile`,
  rodado com a API de pé; o arquivo de configuração do gerador aponta para `http://localhost:3333/api-json`.
- **Uma instância de `dio` só**, em `lib/shared/http/`, com `baseUrl` igual a `API_URL` e o header
  `Clinicore-Client: mobile` em toda requisição (regra 1 da spec irmã). Nenhuma feature cria a sua.
- As rotas são as da tabela da regra 2 da 003-web, mais: `POST /oauth/google/sessions`,
  `POST /passkey-authentication-challenges`, `POST /passkey-assertions`,
  `POST /passkey-registration-challenges`, `POST /passkeys`, `GET /passkeys` e `DELETE /passkeys/{id}`.

### 3. Os tokens ficam no armazenamento seguro, e a renovação é uma por vez

- `flutter_secure_storage` 11.2.0 guarda `accessToken` e `refreshToken` — Keychain no iOS, Keystore no
  Android. Nada de sessão em `SharedPreferences` ou em arquivo.
- Toda requisição autenticada leva `Authorization: Bearer <accessToken>`.
- **Um `401` dispara uma única chamada a `POST /sessions/current/tokens`**, com `{ refreshToken }`;
  as requisições que também tomarem `401` esperam essa mesma chamada. Sucesso grava os dois tokens
  novos e repete a original uma vez. É a regra 11 da 003-web, no interceptor do `dio`.
- **Falha no refresh** apaga os dois tokens e leva para `/login` com "Sua sessão expirou. Entre de
  novo.". `POST /sessions`, `POST /sessions/current/tokens`, `POST /oauth/google/sessions` e
  `POST /passkey-assertions` ficam fora do interceptor.
- **`Sair`** chama `DELETE /sessions/current` e **apaga os tokens do aparelho mesmo que a chamada
  falhe** — sem conexão, sair continua funcionando. Na API, a sessão abandonada vence sozinha em 7 dias.

### 4. Senha, Zod e mensagens: as mesmas regras do web, em Dart

- `lib/features/auth/password_policy.dart` exporta `isStrongPassword`, função pura e testada, com a
  regra 4 da 003-web.
- Toda resposta é validada pelo modelo gerado; resposta que não desserializa cai na mensagem genérica.
- **As mensagens são as da regra 5 da 003-web, palavra por palavra**, traduzidas do `type` do Problem
  Details em `lib/features/auth/api/messages.dart`. Somam-se as da tabela abaixo, que valem para o app
  **e** para o web.

| Código do `type` ou situação | Mensagem exibida |
| --- | --- |
| `invalid-provider-token` | "Não foi possível entrar com o Google. Tente de novo." |
| `unverified-provider-email` | "A sua conta do Google ainda não tem o e-mail confirmado. Confirme no Google e tente de novo." |
| `invalid-passkey-assertion` | "Não foi possível entrar com a biometria. Entre com a senha ou com o Google." |
| `invalid-passkey-registration` | "Não foi possível ativar a biometria. Tente de novo." |
| `passkey-limit-reached` | "Você já tem 10 passkeys. Remova uma para adicionar outra." |
| `passkey-not-found` | "Não foi possível completar a ação. Tente de novo." |
| `invalid-client` | "Não foi possível completar a ação. Tente de novo." |
| A pessoa cancela o Google ou a biometria | nenhuma mensagem |

### 5. Entrar com Google pelo SDK nativo

- `google_sign_in` 7.2.0: `GoogleSignIn.instance.initialize(clientId: GOOGLE_IOS_CLIENT_ID no iOS,
  serverClientId: GOOGLE_SERVER_CLIENT_ID)` e `authenticate()`. O `idToken` vai para
  `POST /oauth/google/sessions`, e a resposta é tratada como a de `POST /sessions`.
- `GOOGLE_SERVER_CLIENT_ID` é o mesmo client ID web que a API tem em `GOOGLE_CLIENT_ID` (regra 5 da
  spec irmã). O iOS registra o client ID invertido em `CFBundleURLTypes`, como o `google_sign_in_ios`
  exige.
- O botão é o mesmo `Entrar com Google` de `/login` e `/signup`.

### 6. Entrar com passkey sem botão próprio

- **Web** — `@simplewebauthn/browser` 14.0.0. Ao abrir `/login`, se
  `browserSupportsWebAuthnAutofill()` for verdadeiro, o web pede
  `POST /passkey-authentication-challenges` e chama `startAuthentication({ optionsJSON,
  useBrowserAutofill: true })`. O campo E-mail passa a ter `autoComplete="username webauthn"` — muda o
  dicionário da 003-web —, e o navegador lista as passkeys salvas ao tocar nele. Escolhida uma, a
  resposta vai para `POST /passkey-assertions` e o usuário segue para `redirect` ou `/app`, como no
  login por senha. Navegador sem suporte simplesmente não oferece nada.
- **App, só Android** — `passkeys` 2.23.1. Ao abrir `/login` e cada vez que o campo E-mail ganha foco,
  o app pede o desafio e chama a autenticação com `preferImmediatelyAvailableCredentials: true`: com
  passkey salva no aparelho, o Android mostra a folha do sistema; sem nenhuma, nada aparece e o
  formulário segue normal.
- **No iOS a passkey não aparece** — nem no login, nem na oferta, nem o link `Biometria e passkeys` —
  até a publicação (Fora de Escopo).
- Cancelar a folha ou o autocompletar não mostra mensagem e não impede o login por senha.

### 7. Ativar a biometria: oferta depois do login e tela da conta

- **A oferta** aparece em `/app` uma vez por aparelho, logo depois de um login por senha ou Google,
  quando o aparelho permite passkey — no web, `platformAuthenticatorIsAvailable()` verdadeiro; no app,
  Android. É um diálogo:
  - título "Entrar com biometria da próxima vez?";
  - texto "Use a digital, o rosto ou o bloqueio de tela deste aparelho para entrar sem digitar a senha.";
  - botões `Ativar` e `Agora não`.
- `Ativar` pede `POST /passkey-registration-challenges`, cria a credencial no aparelho e envia para
  `POST /passkeys`. Sucesso mostra "Biometria ativada neste aparelho.". `Agora não` fecha.
- **Nos dois casos a oferta não volta naquele aparelho**: o web grava `clinicore.passkeyOffer` no
  `localStorage`; o app grava a mesma chave no `flutter_secure_storage`. Quem recusou ativa depois pela
  tela da conta.
- **`/app/account/passkeys`** lista `GET /passkeys`, da mais recente para a mais antiga. Cada item mostra
  "Criada em {dd/mm/aaaa}" e "Último uso em {dd/mm/aaaa}", ou "Ainda não usada" quando `lastUsedAt` é
  nulo. Lista vazia mostra "Nenhuma passkey cadastrada.".
- `Adicionar passkey` faz o mesmo que `Ativar`. No web, com `platformAuthenticatorIsAvailable()` falso,
  o botão não aparece e a tela mostra "Este navegador não permite passkeys.".
- `Remover` pede confirmação — "Remover esta passkey? Você não vai mais entrar com ela." com os botões
  `Remover` e `Cancelar` — e chama `DELETE /passkeys/{id}`. Sucesso mostra "Passkey removida." e tira
  o item da lista.

### 8. Configuração do app

- `lib/shared/env/env.dart` é o único leitor de configuração, por `String.fromEnvironment`, e exige
  `API_URL` (URL absoluta, sem barra final), `GOOGLE_SERVER_CLIENT_ID` e `GOOGLE_IOS_CLIENT_ID`, todos
  não vazios. **Sem valor padrão.** Faltando algum, o app abre numa tela que lista os nomes faltantes,
  em texto de desenvolvedor: "Missing configuration: API_URL".
- Os valores entram por `--dart-define-from-file=config/<ambiente>.json`. `config/*.json` fica no
  `.gitignore`, e `config/example.json` é commitado com os nomes e o formato de cada um.
- **O identificador do app é provisório**, `br.com.clinicore.dev`, só para compilar e testar; o
  definitivo sai com o nome comercial, na publicação.

### 9. Os links do e-mail continuam no web

- Confirmar e-mail e redefinir senha terminam no navegador, pelas telas `/verify-email?token=` e
  `/reset-password` do web, como na 003-web.
- No app, `Esqueci minha senha` só pede o link (`POST /password-resets`), com a mesma mensagem do web:
  "Se este e-mail tiver cadastro, você receberá um link para redefinir a senha.".
- Depois de confirmar ou redefinir no navegador, a pessoa volta ao app e entra. O botão "Abrir o app"
  está Fora de Escopo.

### 10. Mobile-first, carregamento e acessibilidade no app

- Toda tela é verificada primeiro num emulador de 390dp de largura e depois num tablet; no tablet o
  cartão fica centralizado com no máximo 400dp.
- Botão em submissão fica desabilitado e troca o rótulo pelo de progresso — `Entrando…`, `Criando…`,
  `Enviando…`, `Salvando…` — sem mudar de tamanho. A carga da sessão em `/app` usa o mesmo contêiner da
  tela final.
- Todo campo tem rótulo visível e `Semantics` com o rótulo; a mensagem de erro do campo é anunciada, e a
  do formulário fica numa região `liveRegion`. Com fonte do sistema em 200%, nenhum texto é cortado.

### 11. Persistência e Auditoria

- **Fronteira do usuário (visual):** `Entrar`, `Criar conta`, `Entrar com Google`, a escolha de uma
  passkey, `Ativar`, `Adicionar passkey`, `Remover`, `Reenviar link`, `Enviar link`, `Salvar` e `Sair`.
- **Ação do sistema (interna):** o app grava só os dois tokens e a marca da oferta, no armazenamento
  seguro; o web grava só a marca da oferta no `localStorage`. Toda gravação de dado acontece na API.
- **Auditoria:** `N/A` — não há acesso a prontuário; a trilha é a issue #9.

---

## Cenários de Aceite (Gherkin)

Testes de widget no app com `flutter_test` e o cliente da API substituído por um duplo; no web, o padrão
da 003-web, com `features/auth/api` em `jest.fn`. Os cenários 12 e 13 são verificados no aparelho.

### Cenário 1 — Entrar no app leva para o Início e guarda os tokens (caminho feliz, regras 1 e 3)

```gherkin
Dado um visitante em `/login` no app
E que a API responde o login com `user` e `tokens`
Quando ele preenche e-mail e senha e toca `Entrar`
Então `POST /sessions` sai com `Clinicore-Client: mobile` e só `email` e `password` no corpo
E os dois tokens ficam no armazenamento seguro
E ele chega em `/app`, que mostra o nome e o e-mail
```

### Cenário 2 — Erros do login com as mensagens do web (exceção, regra 4)

```gherkin
Dado um visitante em `/login` no app
Quando a API responde `401 INVALID_CREDENTIALS`
Então a tela mostra "E-mail ou senha incorretos." e o campo Senha é limpo
Quando a API responde `403 EMAIL_NOT_VERIFIED`
Então ele vai para `/verify-email` com o e-mail digitado
Quando a API responde um `type` sem tradução
Então a tela mostra "Não foi possível completar a ação. Tente de novo."
```

### Cenário 3 — Cadastro no app e confirmação pelo navegador (caminho feliz, regra 9)

```gherkin
Dado um visitante em `/signup` no app com a senha `Clinica#2026`
Quando ele toca `Criar conta` e a API responde `202`
Então ele chega em `/verify-email` com "Enviamos um link de confirmação para {e-mail}."
E `Reenviar link` fica desabilitado por 60 segundos depois de cada toque
E `Já confirmei, entrar` leva para `/login`
E com `semmaiuscula1!` nenhuma requisição sai e a mensagem de senha fraca aparece
```

### Cenário 4 — Entrar com Google no app (caminho feliz e exceção, regra 5)

```gherkin
Dado um visitante em `/login` no app
Quando ele toca `Entrar com Google` e escolhe uma conta
Então o `idToken` vai para `POST /oauth/google/sessions` e ele chega em `/app`
Quando ele cancela a escolha de conta
Então nenhuma requisição sai e nenhuma mensagem aparece
Quando a API responde `401 INVALID_PROVIDER_TOKEN`
Então a tela mostra "Não foi possível entrar com o Google. Tente de novo."
```

### Cenário 5 — Tela protegida sem sessão e sessão em tela pública (exceção e alternativo, regra 1)

```gherkin
Dado o app sem token guardado
Quando ele abre em `/app/account/password`
Então o conteúdo nunca aparece e ele vai para `/login` guardando o destino
E após entrar chega em `/app/account/password`
Dado o app com sessão válida
Quando `/login` é aberta
Então ele vai para `/app`
```

### Cenário 6 — Três `401` juntos renovam uma vez só (caminho feliz, regra 3)

```gherkin
Dado o app em `/app` com o access token vencido
Quando três requisições respondem `401 INVALID_SESSION`
Então `POST /sessions/current/tokens` é chamado uma vez, com `{ refreshToken }`
E os tokens novos substituem os antigos no armazenamento seguro
E as três requisições são repetidas uma vez cada, sem mensagem
```

### Cenário 7 — Refresh que falha manda para o login (exceção, regra 3)

```gherkin
Dado o app com o refresh recusado pela API
Quando a renovação falha
Então os dois tokens são apagados do armazenamento seguro
E ele vai para `/login` com "Sua sessão expirou. Entre de novo."
E um `401` em `POST /sessions` nunca dispara renovação
```

### Cenário 8 — Sair funciona até sem conexão (caminho feliz e alternativo, regra 3)

```gherkin
Dado o app em `/app`
Quando ele toca `Sair`
Então `DELETE /sessions/current` é chamado, os tokens são apagados e ele vai para `/login`
Quando a chamada falha por falta de conexão
Então os tokens são apagados do mesmo jeito e ele vai para `/login` sem mensagem
```

### Cenário 9 — Recuperar e trocar a senha no app (caminho feliz e exceção, regras 4 e 9)

```gherkin
Dado um visitante em `/forgot-password` no app
Quando ele envia um e-mail válido
Então a tela mostra "Se este e-mail tiver cadastro, você receberá um link para redefinir a senha."
Dado um usuário em `/app/account/password`
Quando a API responde `400 INVALID_PASSWORD`
Então a tela mostra "Senha atual incorreta."
E com as senhas novas diferentes a tela mostra "As senhas não coincidem." e nada é enviado
E com a senha atual correta a tela mostra "Senha alterada."
```

### Cenário 10 — Passkey no autocompletar do web (caminho feliz e exceção, regra 6)

```gherkin
Dado um visitante em `/login` no web, num navegador com suporte a autofill de WebAuthn
Então o campo E-mail tem `autocomplete="username webauthn"`
E o web pediu `POST /passkey-authentication-challenges` ao abrir a tela
Quando ele escolhe uma passkey no autocompletar
Então a resposta vai para `POST /passkey-assertions` e ele chega em `/app`
Quando a API responde `401 INVALID_PASSKEY_ASSERTION`
Então a tela mostra "Não foi possível entrar com a biometria. Entre com a senha ou com o Google."
E num navegador sem suporte nenhum desafio é pedido e o login por senha segue igual
```

### Cenário 11 — A oferta aparece uma vez e cadastra a passkey (caminho feliz, regra 7)

```gherkin
Dado um usuário que acabou de entrar por senha num aparelho que permite passkey e nunca viu a oferta
Então `/app` mostra "Entrar com biometria da próxima vez?"
Quando ele toca `Ativar` e confirma a biometria
Então `POST /passkeys` é chamado e a tela mostra "Biometria ativada neste aparelho."
E entrar de novo nesse aparelho não mostra a oferta
E quem tocou `Agora não` também não vê a oferta de novo nesse aparelho
```

### Cenário 12 — A folha de passkey do Android no login (caminho feliz, regra 6)

```gherkin
Dado um Android com uma passkey do Clinicore salva
Quando `/login` é aberta no app
Então a folha do sistema oferece a passkey
Quando ele confirma com a digital
Então ele chega em `/app`
E num Android sem passkey salva nada aparece e o formulário segue normal
E cancelar a folha não mostra mensagem
```

### Cenário 13 — Nada de passkey no iOS até a publicação (exceção, regra 6)

```gherkin
Dado o app rodando no iOS
Quando `/login` e `/app` são abertas
Então nenhum desafio de passkey é pedido
E a oferta de biometria e o link `Biometria e passkeys` não aparecem
```

### Cenário 14 — Gerenciar passkeys (caminho feliz e exceção, regra 7)

```gherkin
Dado um usuário em `/app/account/passkeys` com duas passkeys, uma nunca usada
Então a lista mostra a mais recente primeiro, com "Criada em" e "Último uso em" ou "Ainda não usada"
Quando ele toca `Remover` e confirma
Então `DELETE /passkeys/{id}` é chamado, o item sai e a tela mostra "Passkey removida."
E `Cancelar` não chama nada
Quando `Adicionar passkey` recebe `409 PASSKEY_LIMIT_REACHED`
Então a tela mostra "Você já tem 10 passkeys. Remova uma para adicionar outra."
E sem passkeys a tela mostra "Nenhuma passkey cadastrada."
```

### Cenário 15 — Configuração ausente não deixa o app seguir (exceção, regra 8)

```gherkin
Dado o app compilado sem `API_URL`
Quando ele abre
Então a tela mostra "Missing configuration: API_URL"
E nenhuma requisição sai
```

### Cenário 16 — As telas do app cabem em 390dp e são acessíveis (caminho feliz, regra 10)

```gherkin
Dado um emulador de 390dp de largura com a fonte do sistema em 200%
Quando as sete telas do app são abertas
Então nenhuma tem rolagem horizontal nem texto cortado
E todo campo tem rótulo lido pelo leitor de tela
E o botão em submissão mantém a largura ao trocar o rótulo
```

---

## Dicionário de Dados de Tela (Campos)

Os campos de `/login`, `/signup`, `/verify-email`, `/forgot-password` e `/app/account/password` são os
da 003-web, com os mesmos limites e validações. No app, o equivalente de `autoComplete` é
`autofillHints`: `AutofillHints.email`, `password`, `newPassword` e `name`. Uma mudança no web:

| Tela | Nome do Campo | Tipo | Habilitado | Obrigatório | Regra / Validação |
| --- | --- | --- | --- | --- | --- |
| `/login` (web) | `E-mail` | Texto (254) | Sim | Sim | Formato de e-mail; `autoComplete="username webauthn"` (era `email`) |

`/app/account/passkeys` não tem campo: lista `createdAt` e `lastUsedAt` de `GET /passkeys`, em
`dd/mm/aaaa` no fuso do aparelho.

## Ações de Tela

As ações das telas que existem na 003-web são as de lá. Somam-se:

| Nome da Ação | Destino / Ação | Regra de Ativação | Mensagens Associadas |
| --- | --- | --- | --- |
| `Entrar com Google` (app) | SDK nativo, depois `POST /oauth/google/sessions`; vai para `redirect` ou `/app` | Sempre habilitado | Erro: tabela da regra 4 |
| Escolher passkey | `POST /passkey-assertions`; vai para `redirect` ou `/app` | Web com autofill de WebAuthn; app no Android | Erro: "Não foi possível entrar com a biometria. Entre com a senha ou com o Google." |
| `Ativar` | `POST /passkey-registration-challenges`, cria a credencial, `POST /passkeys` | Na oferta da regra 7 | Sucesso: "Biometria ativada neste aparelho." · Erro: tabela da regra 4 |
| `Agora não` | Fecha a oferta e grava a marca | Na oferta | — |
| `Adicionar passkey` | O mesmo que `Ativar` | Aparelho que permite passkey | Sucesso: "Biometria ativada neste aparelho." · Erro: tabela da regra 4 |
| `Remover` | Confirmação, depois `DELETE /passkeys/{id}` | Sempre habilitado | Sucesso: "Passkey removida." · Erro: "Não foi possível completar a ação. Tente de novo." |
| `Já confirmei, entrar` | Link para `/login` | Sempre habilitado | — |
| `Biometria e passkeys` | Link para `/app/account/passkeys` | Web, e app no Android | — |

---

## Fora de Escopo

- **Publicação na App Store e no Play, identificador definitivo e ícones do app** — dependem do nome
  comercial (pergunta 1 em aberto de `docs/requirements.md`). É a gêmea da #81 e nasce quando o nome
  for decidido.
- **Passkey no iOS** — exige a conta paga da Apple para o Associated Domains; entra com a publicação.
- **Botão "Abrir o app / instalar" no web** — sem loja não há o que instalar; entra com a publicação,
  já com o aviso de app não instalado e o link da loja.
- **Links do e-mail abrindo o app (Universal Links e App Links)** — decidido manter no web.
- **Trava do app por biometria** — decidido que a biometria é login, não bloqueio (ADR 0007, item 7).
- **Offline, banco local e fila** — spec do motor de sync.
- **Tema, cores, tipografia** — fase de design system, como no web.
- **Nome da passkey ou do aparelho na lista** — não pedidos.

## Quebra em Tasks

A numeração continua a da spec irmã, que fica com as tasks 1 a 3. Cada task do app é a gêmea de uma
issue do web, indicada no título do escopo.

| # | Issue | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- | --- |
| 4 | #95 | Create apps/mobile with green gates, the CI job and the generated API client | Gêmea da #59. Projeto Flutter 3.47 com o identificador provisório; `analysis_options.yaml`; gates `flutter analyze`, `dart format --set-exit-if-changed .` e `flutter test`; job `mobile` no CI; `env.dart` e `config/example.json`; `swagger_parser` e o cliente gerado commitado; `dio` com o header `Clinicore-Client`; `go_router` 18.0.1 com `/`; `apps/mobile/CLAUDE.md`; a coluna, os comandos e a pasta do `apps/mobile` no `CLAUDE.md` da raiz | Cenário 15 verde; os três gates saem com código 0 | 1 |
| 5 | #96 | Add the base components of apps/mobile | Gêmea da #77. Campo com rótulo e erro anunciado, botão com rótulo de progresso de largura fixa, cartão e região de mensagem, em `lib/shared/`, sobre os widgets Material 3 | Cenário 16 verde nos componentes; gates verdes | 4 |
| 6 | #97 | Create the sign-in and sign-up screens on apps/mobile | Gêmea da #78. `/login`, `/signup` e `/verify-email`; `password_policy.dart`; `messages.dart`; Google pelo `google_sign_in` 7.2.0 | Cenários 1, 2, 3 e 4 verdes | 2, 5, #67 |
| 7 | #98 | Protect the application area, refresh the session and sign out on apps/mobile | Gêmea da #79. Guarda do `go_router`, `/app`, `flutter_secure_storage`, o interceptor de renovação e `Sair` | Cenários 5, 6, 7 e 8 verdes | 6 |
| 8 | #99 | Recover and change the password on apps/mobile | Gêmea da #80. `/forgot-password` e `/app/account/password` | Cenário 9 verde; Cenário 16 nas sete telas | 7, #70 |
| 9 | #101 | Sign in with a passkey and manage passkeys on apps/web | `@simplewebauthn/browser` 14.0.0; autofill em `/login`; a oferta em `/app`; `/app/account/passkeys`; as mensagens novas | Cenários 10, 11 e 14 verdes no web | 3, #79 |
| 10 | #102 | Sign in with a passkey and manage passkeys on apps/mobile | Gêmea da 9, só Android. `passkeys` 2.23.1; a folha no login; a oferta; `/app/account/passkeys`; `public/.well-known/assetlinks.json` no `apps/web` com o identificador provisório e o certificado de debug, servido como JSON sem redirecionamento; o hash de debug em `PASSKEY_ANDROID_ORIGINS` de homolog | Cenários 11, 12, 13 e 14 verdes no app, verificados em homolog | 3, 7, 9, #4 |

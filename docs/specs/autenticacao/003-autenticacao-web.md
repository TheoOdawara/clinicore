# 003 — Entrar, cadastrar-se e recuperar a senha (usuário da clínica)

> **Status:** fechada
> **Perfil:** UI
> **Módulo:** `apps/web`
> **Epic:** #1 — Plataforma
> **Issue:** #3
> **Spec irmã:** `docs/specs/autenticacao/003-autenticacao-api.md` (perfil API)
> **Decisões:** `docs/decisions/0002-web-e-site-em-vite.md` ·
> `docs/decisions/0003-design-system-com-shadcn-ui-e-react-bits.md` ·
> `docs/decisions/0006-api-rest-e-problem-details.md`

## Acceptance Criteria

### Referência visual

`N/A` — sem protótipo. O layout é definido pelas regras abaixo, e o tema visual da aplicação não é
esta issue. As telas usam os componentes do shadcn/ui em `shared/`, decididos na ADR 0003, com
utilitárias do Tailwind no que o shadcn não cobrir. **Tema, tipografia e cores continuam fora**: aqui o
shadcn entra pelo que ele já resolve de rótulo, descrição de erro, anúncio e foco — a regra 8.

### Especificação das telas

Sete telas. Cinco públicas, duas dentro da área logada.

| Rota | Tela | Acesso |
| --- | --- | --- |
| `/login` | Entrar | público |
| `/signup` | Criar conta | público |
| `/verify-email` | Confirme seu e-mail | público |
| `/forgot-password` | Recuperar senha | público |
| `/reset-password` | Definir nova senha | público, recebe `token` ou `error` na query |
| `/app` | Área logada | sessão válida |
| `/app/account/password` | Trocar senha | sessão válida |

- **`/` continua pública e inalterada** — a tela atual com o título `Clinicore`. A área logada nasce
  em `/app`, sob `routes/(app)/`.
- **`/login`** — página centralizada, um cartão com os campos E-mail e Senha, o botão `Entrar`, um
  separador com a palavra `ou`, o botão `Entrar com Google`, e dois links: `Esqueci minha senha` e
  `Criar conta`.
- **`/signup`** — mesma estrutura, com os campos Nome, E-mail e Senha, o botão `Criar conta`, o botão
  `Entrar com Google` e o link `Já tenho conta`. Abaixo do campo Senha, a regra de senha fica sempre
  visível como texto auxiliar.
- **`/verify-email`** — é para onde o cadastro redireciona e para onde o link de confirmação aponta,
  com `?token=` (ADR 0006). Três estados, decididos na abertura:
  - **com `token` na query** — chama `POST /email-verifications/confirmation` com `{ token }`. O `204`
    traz a sessão nos cookies e leva para `/app`; o `400` com `type` `invalid-token` ou `token-expired`
    mostra a mensagem de link expirado, o campo E-mail e o botão `Reenviar link`;
  - **com sessão válida e sem `token`** — redireciona para `/app`;
  - **sem sessão e sem `token`** — mostra a instrução, o e-mail para onde o link foi enviado, quando
    ele veio no estado da navegação, e o botão `Reenviar link`. Sem e-mail no estado da navegação,
    mostra o campo E-mail.
- **`/forgot-password`** — um campo E-mail, o botão `Enviar link` e o link `Voltar para entrar`.
- **`/reset-password`** — chega pelo redirecionamento do link de reset da API, com `?token=` ou com
  `?error=INVALID_TOKEN`. Com `token`, mostra os campos Nova senha e Confirmar nova senha, e o botão
  `Redefinir senha`. Com `error`, ou sem nenhum dos dois, mostra o estado de erro e o link
  `Pedir um novo link`.
- **`/app`** — placeholder da aplicação: o nome e o e-mail do usuário logado, o link `Trocar senha` e
  o botão `Sair`. Não existe menu, nem barra lateral, nem qualquer outra funcionalidade.
- **`/app/account/password`** — os campos Senha atual, Nova senha e Confirmar nova senha, o botão
  `Salvar` e o link `Voltar`.

### Caminho de menu

`N/A` — não existe menu nesta entrega. As telas públicas são alcançadas por URL direta e pelos links
entre elas; `/app/account/password` é alcançada pelo link `Trocar senha` em `/app`.

### Perfis e privilégios

| Perfil / Ação | Privilégio | Observação |
| --- | --- | --- |
| Visitante sem sessão | — | Vê `/login`, `/signup`, `/verify-email`, `/forgot-password` e `/reset-password` |
| Usuário autenticado | sessão válida | Vê `/app` e `/app/account/password` |
| Usuário autenticado em tela pública | sessão válida | É redirecionado para `/app` |

**Não existe papel nesta entrega.** Papéis e permissões são as issues #6 e #7.

---

## Regras de Negócio

### 1. Restrição de Acesso

- `routes/(app)/route.tsx` é o único guardião da área logada. No `beforeLoad` ele carrega a sessão com
  `queryClient.ensureQueryData(sessionQueryOptions)`.
- Sem sessão, o processamento é bloqueado e o usuário é redirecionado para `/login`, com o destino
  original guardado em `search.redirect`. Não há mensagem: a tela protegida nunca chega a aparecer.
- Depois de entrar, o usuário volta para `search.redirect` quando ele existe, e para `/app` quando
  não existe.
- **A guarda fica na rota de layout, não em cada tela.** Toda tela da aplicação passa por ela; colocar
  a checagem em cada componente deixaria a próxima tela desprotegida por omissão.
- **`/login` e `/signup` fazem o inverso:** com sessão válida, redirecionam para `/app`.
- **Sessão que vence durante o uso** não passa por aqui: quem trata é o interceptor da regra 11, que
  tenta renovar antes de mandar o usuário para `/login`.

#### Origem do fluxo

O usuário chega por URL direta, pelo link de confirmação recebido por e-mail, ou pelo redirecionamento
da guarda ao tentar abrir uma tela da aplicação sem sessão.

### 2. A API é chamada com credenciais, por uma instância só de axios

- **`shared/http/` cria a única instância de axios do app**, com `baseURL: env.VITE_API_URL` e
  `withCredentials: true`. Nenhuma feature cria instância própria nem importa `axios` direto.
- **Não existe mais cliente do Better Auth.** Ele saiu da API na ADR 0001, e cada ação desta spec passa
  a ser uma chamada às rotas da spec irmã:

  | Ação da tela | Rota |
  | --- | --- |
  | Entrar | `POST /sessions` |
  | Criar conta | `POST /users` |
  | Carregar a sessão | `GET /sessions/current` |
  | Renovar a sessão | `POST /sessions/current/tokens` (regra 11) |
  | Sair | `DELETE /sessions/current` |
  | Reenviar link de confirmação | `POST /email-verifications` |
  | Enviar link de recuperação | `POST /password-resets` |
  | Redefinir a senha | `POST /password-resets/confirmation` |
  | Trocar a senha | `PUT /users/me/password` |
  | Entrar com Google | navegação de topo para `GET /oauth/google` |

- **O web não envia URL de redirecionamento em nenhuma chamada.** `callbackURL`, `errorCallbackURL` e
  `redirectTo` deixaram de existir: a API monta todo destino no servidor a partir de `APP_ORIGIN`, e o
  DTO recusa campo desconhecido. Mandar qualquer um deles faz a requisição ser recusada.
- **`Entrar com Google` é navegação de topo, não requisição.** O botão leva o browser para
  `<VITE_API_URL>/oauth/google` com `window.location.assign`, porque o fluxo termina em `302` para o
  Google e volta por navegação. Chamar essa rota por axios não funcionaria: o `XHR` segue o
  redirecionamento dentro da própria requisição, e o usuário nunca sairia da página.
- **O JavaScript nunca lê o cookie de sessão** — ele é `HttpOnly`. Quem diz se há sessão é
  `GET /sessions/current`, e o resultado é cacheado pelo TanStack Query.
- `sessionQueryOptions` vive em `features/auth/api/` e é o que rota e componentes consomem.

### 3. Toda resposta da API passa por um schema Zod

- `features/auth/api/` declara o schema Zod de cada resposta consumida, replicando à mão o DTO da API,
  e os tipos saem de `z.infer`.
- Uma resposta que não passe no `.parse()` é tratada como erro e cai na mensagem genérica da regra 5.
- `user.id`, `session.id` e `session.userId` são validados com `z.uuid()`: a API usa UUID em toda
  chave primária.

### 4. A regra de senha é a mesma da API, replicada

- `features/auth/password-policy.ts` exporta `isStrongPassword(password: string): boolean` como
  **função pura, testada**, com a mesma regra da API: mínimo 8 caracteres, máximo 128, ao menos uma
  letra maiúscula, ao menos um dígito e ao menos um caractere que não seja letra nem dígito.
- Ela alimenta o schema Zod usado pelo TanStack Form nos validadores de `blur` e de `submit` das telas
  `/signup`, `/reset-password` e `/app/account/password`.
- **A replicação é deliberada.** Os dois apps não compartilham código; a API é a autoridade e recusa
  de novo o que passar daqui. O web valida para dar resposta imediata, nunca como única defesa.
- O campo `Confirmar nova senha` é validado só no web: ele não existe na API.

### 5. O texto que o usuário lê é pt-BR e nasce do `type`

Todo erro da API é Problem Details da RFC 9457 (ADR 0006). `features/auth/api/messages.ts` traduz o
`type` da resposta, identificado pelo código depois do prefixo `tag:clinicore.com.br,2026:`, e o
`error` que o fluxo do Google devolve na query. Um código sem tradução cai na mensagem genérica, e a
resposta original é registrada no console para depuração.

| Código do `type`, `error` da query ou situação | Mensagem exibida |
| --- | --- |
| `invalid-credentials` | "E-mail ou senha incorretos." |
| `email-not-verified` | "Confirme seu e-mail antes de entrar. Confira sua caixa de entrada ou peça um novo link." |
| `invalid-token` | "Este link expirou ou já foi usado. Peça um novo." |
| `token-expired` | "Este link expirou ou já foi usado. Peça um novo." |
| `invalid-password` | "Senha atual incorreta." |
| `invalid-session` | "Sua sessão expirou. Entre de novo." |
| `session-reused` | "Sua sessão expirou. Entre de novo." |
| `INVALID_STATE`, na query | "Não foi possível entrar com o Google. Tente de novo." |
| `UNVERIFIED_PROVIDER_EMAIL`, na query | "A sua conta do Google ainda não tem o e-mail confirmado. Confirme no Google e tente de novo." |
| `PROVIDER_ERROR`, na query | "Não foi possível entrar com o Google. Tente de novo." |
| `service-unavailable` | "O sistema está indisponível no momento. Tente de novo em instantes." |
| `rate-limited`, ou HTTP `429` | "Muitas tentativas. Tente de novo em um minuto." |
| `validation-failed` | "Não foi possível completar a ação. Tente de novo." |
| `about:blank`, qualquer status | "Não foi possível completar a ação. Tente de novo." |
| `invalid-origin` | "Não foi possível completar a ação. Tente de novo." |
| resposta que não passa no `.parse()` do Zod | "Não foi possível completar a ação. Tente de novo." |
| qualquer outro, ou falha de rede | "Não foi possível completar a ação. Tente de novo." |

**`validation-failed` e `invalid-origin` não ganham texto próprio de propósito.** O web valida todo campo
antes de enviar (regras 3 e 4), e a origem é configuração — se qualquer um dos dois chegar, é defeito
nosso, não algo que o usuário possa corrigir. Os dois caem na mensagem genérica e a resposta inteira vai
para o console, como manda o parágrafo acima.

**Não existe `type` de senha fraca na API.** Senha fora da política volta como `validation-failed`, com
`{ "pointer": "#/password", "code": "WEAK_PASSWORD" }` em `errors`. A mensagem de senha fraca que o
usuário lê é escrita no navegador, pela regra 4, antes de qualquer requisição sair.

Mensagens validadas apenas no navegador, sem `type` correspondente na API:

| Situação | Mensagem exibida |
| --- | --- |
| Senha fora da política da regra 4 | "A senha precisa ter no mínimo 8 caracteres, com uma letra maiúscula, um número e um caractere especial." |
| `Confirmar nova senha` diferente de `Nova senha` | "As senhas não coincidem." |
| `Nova senha` igual à `Senha atual` | "A nova senha precisa ser diferente da atual." |
| Campo obrigatório vazio ao submeter | "Preencha este campo." |
| E-mail fora do formato | "Informe um e-mail válido." |

Mensagens de sucesso:

| Ação | Mensagem exibida |
| --- | --- |
| Cadastro concluído | "Enviamos um link de confirmação para {e-mail}." |
| Link de confirmação pedido | "Se houver uma confirmação pendente para este e-mail, você receberá um novo link." |
| Recuperação solicitada | "Se este e-mail tiver cadastro, você receberá um link para redefinir a senha." |
| Senha redefinida | "Senha redefinida. Entre com a nova senha." |
| Senha alterada | "Senha alterada." |

**Cadastro com e-mail já cadastrado não tem tratamento no web.** A API responde `202` com corpo vazio,
exatamente como num cadastro novo, e não envia e-mail (regra 8 da spec irmã). O web mostra a mesma
mensagem de sucesso e leva para `/verify-email` nos dois casos, sem ter como distingui-los.

**As mensagens de reenvio e de recuperação não afirmam que um e-mail saiu.** A API limita pedidos por
endereço (regra 15 da spec irmã) e responde igual quando barra; uma frase como "Link reenviado" seria
falsa nesse caso.

### 6. Mobile-first

- Cada tela é desenhada em 390px de largura e só depois expandida para desktop.
- A verificação abre a **device toolbar do DevTools no perfil de celular antes do desktop** —
  redimensionar a janela não vale, porque perde viewport móvel, DPR e emulação de toque.
- Nenhuma tela tem rolagem horizontal em 390px.
- O cartão de formulário ocupa a largura total menos 16px de margem em cada lado no celular, e no
  máximo 400px a partir de 640px.

### 7. O estado de carregamento ocupa o espaço do conteúdo final

- Enquanto uma submissão está em curso, o botão fica desabilitado e troca o rótulo pelo texto de
  progresso — **sem mudar de tamanho**, porque o rótulo de progresso reserva a mesma largura.
- `routes/(app)/route.tsx` declara `pendingComponent` e `errorComponent`. O `pendingComponent`
  reutiliza o mesmo contêiner da tela final, de modo que nada se move quando a sessão chega.
- Verificação: `CLS` igual a 0 e comparação visual entre carregando e carregado.

### 8. Acessibilidade mínima

- Todo campo tem `label` associado, nunca só `placeholder`.
- A mensagem de erro de um campo é referenciada por `aria-describedby` e anunciada com `role="alert"`.
- A mensagem de erro ou de sucesso do formulário fica numa região com `aria-live="polite"`.
- O foco é visível em todo elemento interativo, e a ordem de tabulação segue a ordem visual.

### 9. O web é um PWA instalável

- `vite-plugin-pwa` 1.3.0, que declara suporte ao Vite 8, com `registerType: "autoUpdate"` e a
  estratégia padrão `generateSW`. O Vite não tem PWA nativo, e um service worker escrito à mão teria
  de refazer a lista de arquivos com hash a cada build.
- O build gera `manifest.webmanifest` e `sw.js`, nomes padrão do plugin. O registro do service worker
  é o padrão do plugin, sem código próprio em `main.tsx`.
- O manifesto:

  | Campo | Valor |
  | --- | --- |
  | `name` | `Clinicore` |
  | `short_name` | `Clinicore` |
  | `lang` | `pt-BR` |
  | `display` | `standalone` |
  | `start_url` | `/app` |
  | `scope` | `/` |
  | `icons` | `/pwa-192x192.png` (192×192), `/pwa-512x512.png` (512×512) e `/maskable-icon-512x512.png` (512×512, `purpose: "maskable"`) |

- **Os três ícones não são desta issue.** Eles saem da fase de design system e são entregues em
  `apps/web/public/`, com os nomes e tamanhos da tabela. `theme_color` e `background_color` ficam fora
  do manifesto até essa fase definir as cores; nenhum dos dois é exigido para instalar.
- **O service worker guarda só os arquivos estáticos do próprio build**: `globPatterns` igual a
  `["**/*.{js,css,html,png,svg,webmanifest}"]` e `navigateFallback` igual a `index.html`.
  **Nenhuma resposta da API é guardada**: não existe `runtimeCaching`, e a API fica em outra origem,
  fora do alcance do precache. Dado de saúde nunca fica em cache num dispositivo compartilhado da
  clínica.
- Uma versão nova é ativada sozinha na próxima abertura, sem aviso na tela.
- Sem conexão, as telas abrem a partir do cache e toda chamada à API cai na mensagem genérica da
  regra 5.

### 10. Persistência e Auditoria

- **Fronteira do usuário (visual):** os botões `Entrar`, `Criar conta`, `Entrar com Google`,
  `Reenviar link`, `Enviar link`, `Redefinir senha`, `Salvar` e `Sair`.
- **Ação do sistema (interna):** o web **não persiste nada**. Toda gravação acontece na API, descrita
  na regra 12 da spec irmã. O único estado do web é o cache do TanStack Query, invalidado a cada
  mudança de sessão.
- **Auditoria:** `N/A` — a trilha de auditoria é a issue #9, e não há acesso a prontuário aqui.

### 11. A sessão se renova sozinha, uma vez por vez

O access token vive 15 minutos e o refresh vive 24 horas (regra 2 da spec irmã). O Better Auth escondia
essa renovação; agora ela é código nosso, e mora inteira no interceptor de resposta da instância de
`shared/http/`.

- **Um `401` dispara uma única chamada a `POST /sessions/current/tokens`.** Enquanto ela está em curso, toda
  requisição que também tomar `401` **espera essa mesma chamada** em vez de disparar a sua. Sem isso,
  uma tela que carrega três recursos de uma vez dispara três refreshes, e a rotação da regra 3 da spec
  irmã trata o segundo como reúso e **derruba a sessão do usuário**.
- **Sucesso:** a requisição original é repetida **uma vez**, com os cookies novos. Um segundo `401` na
  repetição não tenta renovar de novo.
- **Falha:** o cache do TanStack Query é invalidado, o usuário vai para `/login` com o destino atual em
  `search.redirect`, e a mensagem exibida é a de `INVALID_SESSION` da regra 5: "Sua sessão expirou.
  Entre de novo.".
- **`POST /sessions/current/tokens` e `POST /sessions` estão fora do interceptor.** Um `401` neles é resposta
  legítima, não sessão vencida; tentar renovar a partir deles seria laço infinito.
- **O web não sabe quando o token expira** — ele não lê o cookie, que é `HttpOnly`, e não há
  temporizador. A renovação é sempre reativa, disparada por um `401`.

---

## Cenários de Aceite (Gherkin)

Os testes cobrem lógica pura e componente que decide algo, com `features/auth/api` substituído por
`jest.fn`, no padrão do `CLAUDE.md`. Os cenários 6 e 12 são verificados no navegador, não em suíte.

### Cenário 1 — Entrar leva para a área logada (caminho feliz, regra 1)

```gherkin
Dado um visitante em `/login`
E que a API responde o login com sucesso
Quando ele preenche e-mail e senha e aciona `Entrar`
Então o formulário é submetido uma única vez
E o corpo de `POST /sessions` traz só `email` e `password`, sem nenhuma URL de redirecionamento
E o usuário é levado para `/app`
E `/app` exibe o nome e o e-mail devolvidos pela sessão
```

### Cenário 2 — Senha errada mostra a mensagem em pt-BR (exceção, regra 5)

```gherkin
Dado um visitante em `/login`
E que a API responde `401` com o código `INVALID_CREDENTIALS`
Quando ele aciona `Entrar`
Então a tela exibe "E-mail ou senha incorretos."
E a mensagem está numa região com `aria-live="polite"`
E o usuário continua em `/login`
E o campo Senha é limpo
```

### Cenário 3 — E-mail não verificado explica o que fazer (caminho alternativo, regra 5)

```gherkin
Dado um visitante em `/login`
E que a API responde `403` com o código `EMAIL_NOT_VERIFIED`
Quando ele aciona `Entrar`
Então a tela exibe "Confirme seu e-mail antes de entrar. Confira sua caixa de entrada ou peça um novo link."
E o usuário é levado para `/verify-email` com o e-mail digitado no estado da navegação
```

### Cenário 4 — Cadastro leva para a confirmação, com e-mail novo ou já usado (caminho feliz, regra 5)

```gherkin
Dado um visitante em `/signup`
E que a API responde `202` com o corpo vazio
Quando ele aciona `Criar conta`
Então o corpo de `POST /users` traz só `name`, `email` e `password`
E a tela exibe "Enviamos um link de confirmação para {e-mail}." com o e-mail digitado
E o usuário é levado para `/verify-email`
E o componente não recebe nenhuma informação que distinga um e-mail novo de um já cadastrado
```

### Cenário 5 — Senha fraca é barrada antes de sair do navegador (exceção, regra 4)

```gherkin
Dado um visitante em `/signup`
Quando ele digita `semmaiuscula1!` no campo Senha e sai do campo
Então a tela exibe "A senha precisa ter no mínimo 8 caracteres, com uma letra maiúscula, um número e um caractere especial."
E o botão `Criar conta` não dispara nenhuma requisição enquanto a senha for inválida
E `Clinica#2026` limpa a mensagem e libera o botão
```

### Cenário 6 — As telas cabem em 390px (caminho feliz, regra 6)

```gherkin
Dado o DevTools com a device toolbar num perfil de celular de 390px de largura
Quando `/login`, `/signup`, `/verify-email`, `/forgot-password`, `/reset-password`, `/app` e `/app/account/password` são abertas
Então nenhuma delas tem rolagem horizontal
E todo rótulo e toda mensagem ficam legíveis sem corte
E só depois disso a verificação é repetida em desktop
```

### Cenário 7 — Tela protegida sem sessão redireciona guardando o destino (exceção, regra 1)

```gherkin
Dado um visitante sem sessão
Quando ele abre `/app/account/password`
Então o conteúdo da tela nunca é renderizado
E ele é levado para `/login` com `redirect` igual a `/app/account/password` na query
E após entrar com sucesso ele chega em `/app/account/password`
```

### Cenário 8 — Sessão válida em tela pública vai para a aplicação (caminho alternativo, regra 1)

```gherkin
Dado um usuário com sessão válida
Quando ele abre `/login`
Então ele é levado para `/app`
E o formulário de login não é renderizado
```

### Cenário 9 — Sair limpa a sessão e o cache (caminho feliz, regra 2)

```gherkin
Dado um usuário em `/app`
Quando ele aciona `Sair`
Então a API recebe a chamada de logout
E o cache do TanStack Query é invalidado
E ele é levado para `/login`
E voltar no histórico do navegador não mostra `/app` com os dados anteriores
```

### Cenário 10 — Link de redefinição inválido oferece uma saída (exceção, regra 5)

```gherkin
Dado um visitante que abre `/reset-password?error=INVALID_TOKEN`
Então a tela exibe "Este link expirou ou já foi usado. Peça um novo."
E oferece o link `Pedir um novo link` para `/forgot-password`
E os campos de senha não são renderizados
E o mesmo acontece em `/reset-password` sem `token` e sem `error`
E o mesmo acontece quando o `POST` de redefinição responde com o `type` `invalid-token`
```


### Cenário 11 — Excesso de tentativas é explicado (exceção, regra 5)

```gherkin
Dado um visitante em `/login`
E que a API responde `429`
Quando ele aciona `Entrar`
Então a tela exibe "Muitas tentativas. Tente de novo em um minuto."
```

### Cenário 12 — Nada se move quando a sessão carrega (caminho feliz, regra 7)

```gherkin
Dado `/app` sendo aberta com a sessão ainda carregando
Quando a sessão chega
Então nenhum elemento da tela muda de posição
E o `CLS` medido é 0
E o botão em submissão mantém a mesma largura ao trocar o rótulo pelo texto de progresso
```

### Cenário 13 — Entrar com Google sai para o provedor (caminho alternativo)

```gherkin
Dado um visitante em `/login`
Quando ele aciona `Entrar com Google`
Então o browser navega para `<VITE_API_URL>/oauth/google`, sem passar por axios
E nenhuma credencial é enviada pelo formulário
E nenhuma URL de redirecionamento é enviada na query
E o mesmo botão em `/signup` faz exatamente a mesma navegação
Quando o Google devolve o visitante para `/login?error=INVALID_STATE`
Então a tela exibe "Não foi possível entrar com o Google. Tente de novo."
E com `error=UNVERIFIED_PROVIDER_EMAIL` exibe a mensagem correspondente da regra 5
```

### Cenário 14 — Trocar a senha exige a senha atual (exceção, regra 5)

```gherkin
Dado um usuário em `/app/account/password`
E que a API responde `400` com o código `INVALID_PASSWORD`
Quando ele aciona `Salvar`
Então a tela exibe "Senha atual incorreta."
E ele continua logado e na mesma tela
E com a senha atual correta a tela exibe "Senha alterada."
```

### Cenário 15 — Confirmar nova senha precisa bater (exceção, regra 4)

```gherkin
Dado um usuário em `/app/account/password` ou um visitante em `/reset-password`
Quando `Nova senha` e `Confirmar nova senha` são diferentes
Então a tela exibe "As senhas não coincidem."
E nenhuma requisição é enviada
```

### Cenário 16 — Um `type` desconhecido não quebra a tela (exceção, regra 5)

```gherkin
Dado que a API responde com um `type` que não tem tradução
Quando qualquer formulário desta spec é submetido
Então a tela exibe "Não foi possível completar a ação. Tente de novo."
E a resposta original é registrada no console
E a tela continua utilizável
```

### Cenário 17 — Resposta fora do schema é tratada como erro (exceção, regra 3)

```gherkin
Dado que a API responde `200` com um corpo que não passa no `.parse()` do schema Zod da sessão
Quando a tela que consome essa resposta é renderizada
Então nenhum dado inválido chega ao componente
E a tela exibe "Não foi possível completar a ação. Tente de novo."
E o erro do Zod é registrado no console
```

### Cenário 18 — Os formulários são operáveis por teclado e leitor de tela (caminho feliz, regra 8)

```gherkin
Dado o formulário de `/login`
Então cada campo tem um `label` associado ao seu `id`
E navegar por Tab percorre E-mail, Senha, `Entrar`, `Entrar com Google`, `Esqueci minha senha` e `Criar conta`, nessa ordem
E o elemento focado tem contorno de foco visível
Quando um campo fica inválido
Então a mensagem do campo é referenciada pelo `aria-describedby` do campo
E a mensagem tem `role="alert"`
E o mesmo vale para os formulários de `/signup`, `/forgot-password`, `/reset-password` e `/app/account/password`
```

### Cenário 19 — Link de confirmação volta para a tela certa (caminho alternativo, regra 1)

```gherkin
Dado um visitante que abre `/verify-email?token=<token válido>`
Então `POST /email-verifications/confirmation` é chamado só com `token`
E ele é levado para `/app`
Quando ele abre `/verify-email` sem `token` e a sessão carregada é válida
Então ele é levado para `/app`
Quando ele abre `/verify-email?token=<token expirado>` e a API responde `400` com o `type` `token-expired`
Então a tela exibe "Este link expirou ou já foi usado. Peça um novo."
E exibe o campo E-mail e o botão `Reenviar link`
Quando ele aciona `Reenviar link` com um e-mail válido
Então `POST /email-verifications` é chamado só com `email`
E a tela exibe "Se houver uma confirmação pendente para este e-mail, você receberá um novo link."
E o botão fica desabilitado por 60 segundos
```

### Cenário 20 — O build gera um PWA instalável (caminho feliz, regra 9)

```gherkin
Dado os três ícones da regra 9 presentes em `apps/web/public/`
Quando `vite build` é executado
Então `dist/manifest.webmanifest` existe com `name` igual a `Clinicore`, `display` igual a `standalone`, `start_url` igual a `/app` e `lang` igual a `pt-BR`
E lista os três ícones com os tamanhos da regra 9
E `dist/sw.js` existe
E aberto no Chrome, o painel Application do DevTools mostra o app como instalável, sem erro no manifesto
```

### Cenário 21 — Nenhuma resposta da API fica no cache (exceção, regra 9)

```gherkin
Dado o web servido pelo build, com o service worker ativo, e um usuário que entrou e abriu `/app`
Quando a conexão é cortada no DevTools e `/app` é recarregada
Então a tela abre a partir do cache
E a chamada de sessão à API falha em vez de ser respondida pelo service worker
E a tela exibe "Não foi possível completar a ação. Tente de novo."
E o Cache Storage não contém nenhuma URL da origem de `VITE_API_URL`
```

### Cenário 22 — A sessão se renova uma vez só, mesmo com três chamadas juntas (caminho feliz, regra 11)

```gherkin
Dado um usuário em `/app` cujo access token venceu
E uma tela que dispara três requisições à API ao abrir
Quando as três respondem `401` com o código `INVALID_SESSION`
Então `POST /sessions/current/tokens` é chamado exatamente uma vez
E as outras duas esperam essa chamada em vez de dispararem a sua
E, com o refresh bem-sucedido, as três requisições originais são repetidas uma única vez cada
E o usuário permanece em `/app`, sem ver nenhuma mensagem
```

### Cenário 23 — Refresh que falha manda para o login guardando o destino (exceção, regra 11)

```gherkin
Dado um usuário em `/app/account/password` cujo access token venceu
E que `POST /sessions/current/tokens` responde `401`
Quando a renovação falha
Então o cache do TanStack Query é invalidado
E o usuário é levado para `/login` com `redirect` igual a `/app/account/password`
E a tela exibe "Sua sessão expirou. Entre de novo."
E nenhuma segunda chamada a `POST /sessions/current/tokens` é disparada
E um `401` em `POST /sessions` nunca dispara renovação
```

---

## Dicionário de Dados de Tela (Campos)

### `/login`

| Nome do Campo | Tipo | Habilitado | Obrigatório | Regra / Validação |
| --- | --- | --- | --- | --- |
| `E-mail` | Texto (254) | Sim | Sim | Formato de e-mail; `autoComplete="email"`; `inputMode="email"`; recebe o foco ao abrir a tela |
| `Senha` | Senha (128) | Sim | Sim | 1 a 128 caracteres; `autoComplete="current-password"`; sem validação de força, para não dar pista sobre a senha cadastrada |

### `/signup`

| Nome do Campo | Tipo | Habilitado | Obrigatório | Regra / Validação |
| --- | --- | --- | --- | --- |
| `Nome` | Texto (100) | Sim | Sim | 1 a 100 caracteres após remover espaços das pontas; `autoComplete="name"` |
| `E-mail` | Texto (254) | Sim | Sim | Formato de e-mail; `autoComplete="email"` |
| `Senha` | Senha (128) | Sim | Sim | `isStrongPassword` da regra 4; `autoComplete="new-password"`; texto auxiliar fixo: "Mínimo de 8 caracteres, com uma letra maiúscula, um número e um caractere especial." |

### `/verify-email`

| Nome do Campo | Tipo | Habilitado | Obrigatório | Regra / Validação |
| --- | --- | --- | --- | --- |
| `E-mail` | Texto (254) | Condicional (a confirmação do `token` falhou, ou não há e-mail no estado da navegação) | Condicional (quando visível) | Formato de e-mail; `autoComplete="email"`. Quando oculto, o e-mail vem do estado da navegação |

### `/forgot-password`

| Nome do Campo | Tipo | Habilitado | Obrigatório | Regra / Validação |
| --- | --- | --- | --- | --- |
| `E-mail` | Texto (254) | Sim | Sim | Formato de e-mail; `autoComplete="email"` |

### `/reset-password`

| Nome do Campo | Tipo | Habilitado | Obrigatório | Regra / Validação |
| --- | --- | --- | --- | --- |
| `Nova senha` | Senha (128) | Condicional (há `token` e não há `error` na query) | Sim | `isStrongPassword` da regra 4; `autoComplete="new-password"` |
| `Confirmar nova senha` | Senha (128) | Condicional (há `token` e não há `error` na query) | Sim | Igual a `Nova senha`; validado só no web |

### `/app`

Sem campo. Exibe `user.name` e `user.email` da sessão.

### `/app/account/password`

| Nome do Campo | Tipo | Habilitado | Obrigatório | Regra / Validação |
| --- | --- | --- | --- | --- |
| `Senha atual` | Senha (128) | Sim | Sim | 1 a 128 caracteres; `autoComplete="current-password"` |
| `Nova senha` | Senha (128) | Sim | Sim | `isStrongPassword` da regra 4; diferente da senha atual; `autoComplete="new-password"` |
| `Confirmar nova senha` | Senha (128) | Sim | Sim | Igual a `Nova senha`; validado só no web |

## Ações de Tela

| Nome da Ação | Destino / Ação | Regra de Ativação | Mensagens Associadas |
| --- | --- | --- | --- |
| `Entrar` | `POST /sessions` com `email` e `password`; vai para `search.redirect` ou `/app` | Habilitado com e-mail e senha preenchidos; desabilitado durante a submissão, com o rótulo `Entrando…` | Erro: tabela da regra 5 |
| `Entrar com Google` | navegação de topo para `<VITE_API_URL>/oauth/google`, sem corpo e sem query | Sempre habilitado | Erro: o que voltar em `?error=` na query de `/login`, pela tabela da regra 5 |
| `Criar conta` | `POST /users` com `name`, `email` e `password`; vai para `/verify-email` | Habilitado com os três campos válidos; desabilitado durante a submissão, com o rótulo `Criando…` | Sucesso: "Enviamos um link de confirmação para {e-mail}." · Erro: tabela da regra 5 |
| `Reenviar link` | `POST /email-verifications` com `email` | Habilitado com um e-mail conhecido ou digitado; volta a ficar habilitado 60 segundos após cada acionamento | Sucesso: "Se houver uma confirmação pendente para este e-mail, você receberá um novo link." · Erro: "Muitas tentativas. Tente de novo em um minuto." |
| `Enviar link` | `POST /password-resets` com `email`; permanece em `/forgot-password` | Habilitado com o e-mail válido; desabilitado durante a submissão, com o rótulo `Enviando…` | Sucesso: "Se este e-mail tiver cadastro, você receberá um link para redefinir a senha." |
| `Redefinir senha` | `POST /password-resets/confirmation` com o `token` da query e a nova senha; vai para `/login` | Habilitado com as duas senhas válidas e iguais; desabilitado durante a submissão, com o rótulo `Salvando…` | Sucesso: "Senha redefinida. Entre com a nova senha." · Erro: tabela da regra 5 |
| `Salvar` | `PUT /users/me/password` com `currentPassword` e `newPassword`; permanece na tela | Habilitado com os três campos válidos; desabilitado durante a submissão, com o rótulo `Salvando…` | Sucesso: "Senha alterada." · Erro: tabela da regra 5 |
| `Sair` | `DELETE /sessions/current`, invalida o cache do Query; vai para `/login` | Sempre habilitado | Erro: "Não foi possível completar a ação. Tente de novo." |
| `Esqueci minha senha` | Link para `/forgot-password` | Sempre habilitado | — |
| `Criar conta` (link) | Link para `/signup` | Sempre habilitado | — |
| `Já tenho conta` | Link para `/login` | Sempre habilitado | — |
| `Trocar senha` | Link para `/app/account/password` | Sempre habilitado | — |
| `Pedir um novo link` | Link para `/forgot-password` | Visível só no estado de token inválido | — |
| `Voltar` / `Voltar para entrar` | Link para `/app` e para `/login`, respectivamente | Sempre habilitado | — |

---

## Fora de Escopo

- **Tema visual, tipografia, ícones e cores do PWA** — pertencem à fase de design system, que a ADR 0003
  abre sem cobrir. As telas desta spec usam os componentes do shadcn/ui já adotados, com utilitárias do
  Tailwind no resto, e nenhuma decisão de cor ou de tipografia é tomada aqui.
- **Animação e componente do React Bits** — a ADR 0003 define em quais categorias ele pode entrar no
  `apps/web`; nenhuma tela desta spec usa nenhuma delas.
- **Página offline e aviso de nova versão** — não pedidos; a regra 9 define o comportamento sem conexão
  e a atualização automática.
- **Layout da aplicação: menu, barra lateral, cabeçalho** — `/app` é um placeholder até a issue #6
  trazer a primeira tela de verdade.
- **Seletor de clínica, troca de tenant, exibição de papel** — issues #6 e #7.
- **Editar perfil, trocar e-mail, enviar foto, excluir conta** — não pedidos.
- **Sessões ativas por dispositivo e 2FA** — não pedidos.
- **Medidor de força de senha e revelar/ocultar senha** — não pedidos; a regra aparece como texto
  auxiliar fixo.
- **Internacionalização** — a interface é pt-BR, e não há mecanismo de idioma.
- **`/` como tela protegida** — decidido que `/` continua pública e inalterada.

## Quebra em Tasks

A numeração continua a da spec irmã, `docs/specs/autenticacao/003-autenticacao-api.md`, que fica com as tasks 1 a 7.

| # | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- |
| 8 | Create the sign-in and sign-up screens on apps/web | `@tanstack/react-query`, `@tanstack/react-form`, `axios` e `@testing-library/*` instalados, `setupFilesAfterEnv` no `jest.config.json`, `QueryClientProvider` e o contexto do router em `main.tsx`, `shared/http/` com a instância de axios, `features/auth/api/` com schemas Zod e `messages.ts`, `features/auth/password-policy.ts`, rotas `/login`, `/signup` e `/verify-email` | Cenários 1, 2, 3, 4, 5, 11, 13, 16, 17, 18 e 19 verdes; os quatro gates do web saem com código 0 | task de fundação do shadcn/ui (ADR 0003), #65, #66, #67 |
| 9 | Protect the application area, refresh the session and sign out | `routes/(app)/route.tsx` com a guarda, `pendingComponent` e `errorComponent`, rota `/app`, ação `Sair`, o interceptor de renovação da regra 11 em `shared/http/`, `VITE_API_URL` no job `web` do CI | Cenários 7, 8, 9, 12, 22 e 23 verdes | 8 |
| 10 | Recover, reset and change the password on apps/web | Rotas `/forgot-password`, `/reset-password` e `/app/account/password` | Cenários 10, 14 e 15 verdes | 9, #67, #70 |
| 11 | Make apps/web an installable PWA | `vite-plugin-pwa` em `vite.config.ts`, manifesto da regra 9, ícones em `apps/web/public/` | Cenários 20 e 21 verdes | — |

A task 11 não depende de outra task desta spec, mas só fecha quando a fase de design system entregar
os três ícones da regra 9.

O cenário 6, de largura de 390px, é verificado nas tasks 8, 9 e 10, sobre as telas que cada uma
entrega.

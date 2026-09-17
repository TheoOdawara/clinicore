# 003 — Entrar, cadastrar-se e recuperar a senha (usuário da clínica)

> **Status:** fechada
> **Perfil:** UI
> **Módulo:** `apps/web`
> **Epic:** #1 — Plataforma
> **Issue:** #3
> **Spec irmã:** `docs/specs/003-autenticacao-api.md` (perfil API)

## Acceptance Criteria

### Referência visual

`N/A` — sem protótipo. O layout é definido pelas regras abaixo, e o tema visual da aplicação não é
esta issue. As telas usam utilitárias do Tailwind diretamente, sem design system.

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
- **`/verify-email`** — é para onde o cadastro redireciona e para onde o link de confirmação volta.
  Três estados, decididos na abertura:
  - **com sessão válida** — a confirmação deu certo e a API já criou a sessão; redireciona para `/app`;
  - **com `error` na query** (`INVALID_TOKEN` ou `TOKEN_EXPIRED`, colocado pela API) — mostra a
    mensagem de link expirado, o campo E-mail e o botão `Reenviar link`;
  - **sem sessão e sem `error`** — mostra a instrução, o e-mail para onde o link foi enviado, quando
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

#### Origem do fluxo

O usuário chega por URL direta, pelo link de confirmação recebido por e-mail, ou pelo redirecionamento
da guarda ao tentar abrir uma tela da aplicação sem sessão.

### 2. A API é chamada com credenciais

- O cliente do Better Auth é criado uma vez em `shared/auth/`, com
  `baseURL: env.VITE_API_URL` e `fetchOptions: { credentials: "include" }`.
- **Toda URL de redirecionamento enviada à API é absoluta**, montada como
  `${window.location.origin}/<caminho>`. Um caminho relativo é resolvido pelo browser contra o host
  da API, porque é a API que redireciona, e o usuário cairia fora do web. Isso vale para
  `callbackURL`, `errorCallbackURL` e `redirectTo`.
- `shared/http/` centraliza qualquer `fetch` que não passe pelo cliente do Better Auth, também com
  `credentials: "include"`.
- **O JavaScript nunca lê o cookie de sessão** — ele é `httpOnly`. Quem diz se há sessão é
  `getSession`, e o resultado é cacheado pelo TanStack Query.
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

### 5. O texto que o usuário lê é pt-BR e nasce do `code`

`features/auth/api/messages.ts` traduz o `code` da resposta. Um `code` sem tradução cai na mensagem
genérica, e a resposta original é registrada no console para depuração.

| `code` ou situação | Mensagem exibida |
| --- | --- |
| `INVALID_EMAIL_OR_PASSWORD` | "E-mail ou senha incorretos." |
| `EMAIL_NOT_VERIFIED` | "Confirme seu e-mail antes de entrar. Confira sua caixa de entrada ou peça um novo link." |
| `WEAK_PASSWORD` | "A senha precisa ter no mínimo 8 caracteres, com uma letra maiúscula, um número e um caractere especial." |
| `INVALID_TOKEN` | "Este link expirou ou já foi usado. Peça um novo." |
| `TOKEN_EXPIRED` | "Este link expirou ou já foi usado. Peça um novo." |
| `INVALID_PASSWORD` | "Senha atual incorreta." |
| `SESSION_EXPIRED` | "Sua sessão expirou. Entre de novo." |
| HTTP `429` | "Muitas tentativas. Tente de novo em um minuto." |
| resposta que não passa no `.parse()` do Zod | "Não foi possível completar a ação. Tente de novo." |
| qualquer outro, ou falha de rede | "Não foi possível completar a ação. Tente de novo." |

Mensagens validadas apenas no navegador, sem `code` correspondente na API:

| Situação | Mensagem exibida |
| --- | --- |
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

**Cadastro com e-mail já cadastrado não tem tratamento no web.** A API responde `200` com `token`
nulo, exatamente como num cadastro novo, e não envia e-mail (regra 8 da spec irmã). O web mostra a
mesma mensagem de sucesso e leva para `/verify-email` nos dois casos, sem ter como distingui-los.

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
E o cliente é chamado com `callbackURL` igual a `${window.location.origin}/verify-email`
E o usuário é levado para `/app`
E `/app` exibe o nome e o e-mail devolvidos pela sessão
```

### Cenário 2 — Senha errada mostra a mensagem em pt-BR (exceção, regra 5)

```gherkin
Dado um visitante em `/login`
E que a API responde `401` com o código `INVALID_EMAIL_OR_PASSWORD`
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
E que a API responde `200` com `token` nulo
Quando ele aciona `Criar conta`
Então o cliente é chamado com `callbackURL` igual a `${window.location.origin}/verify-email`
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
E o mesmo acontece quando o `POST` de redefinição responde `INVALID_TOKEN`
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
Então o cliente do Better Auth é chamado com o provedor `google`, `callbackURL` igual a `${window.location.origin}/app` e `errorCallbackURL` igual a `${window.location.origin}/login`
E nenhuma credencial é enviada pelo formulário
E o mesmo botão em `/signup` faz exatamente a mesma chamada
Quando o Google devolve o visitante para `/login` com `error` na query
Então a tela exibe "Não foi possível completar a ação. Tente de novo."
```

### Cenário 14 — Trocar a senha exige a senha atual (exceção, regra 5)

```gherkin
Dado um usuário em `/app/account/password`
E que a API responde `401` com o código `INVALID_PASSWORD`
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

### Cenário 16 — Um `code` desconhecido não quebra a tela (exceção, regra 5)

```gherkin
Dado que a API responde com um `code` que não tem tradução
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
Dado um visitante que abre `/verify-email`
Quando a sessão carregada é válida
Então ele é levado para `/app`
Quando não há sessão e a query traz `error=TOKEN_EXPIRED`
Então a tela exibe "Este link expirou ou já foi usado. Peça um novo."
E exibe o campo E-mail e o botão `Reenviar link`
Quando ele aciona `Reenviar link` com um e-mail válido
Então o cliente é chamado com `callbackURL` igual a `${window.location.origin}/verify-email`
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
| `E-mail` | Texto (254) | Condicional (há `error` na query, ou não há e-mail no estado da navegação) | Condicional (quando visível) | Formato de e-mail; `autoComplete="email"`. Quando oculto, o e-mail vem do estado da navegação |

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
| `Entrar` | `signIn.email` do cliente com `callbackURL` igual a `${window.location.origin}/verify-email`; vai para `search.redirect` ou `/app` | Habilitado com e-mail e senha preenchidos; desabilitado durante a submissão, com o rótulo `Entrando…` | Erro: tabela da regra 5 |
| `Entrar com Google` | `signIn.social` com `provider: "google"`, `callbackURL` igual a `${window.location.origin}/app` e `errorCallbackURL` igual a `${window.location.origin}/login` | Sempre habilitado | Erro: "Não foi possível completar a ação. Tente de novo." |
| `Criar conta` | `signUp.email` com `callbackURL` igual a `${window.location.origin}/verify-email`; vai para `/verify-email` | Habilitado com os três campos válidos; desabilitado durante a submissão, com o rótulo `Criando…` | Sucesso: "Enviamos um link de confirmação para {e-mail}." · Erro: tabela da regra 5 |
| `Reenviar link` | `sendVerificationEmail` com `callbackURL` igual a `${window.location.origin}/verify-email` | Habilitado com um e-mail conhecido ou digitado; volta a ficar habilitado 60 segundos após cada acionamento | Sucesso: "Se houver uma confirmação pendente para este e-mail, você receberá um novo link." · Erro: "Muitas tentativas. Tente de novo em um minuto." |
| `Enviar link` | `requestPasswordReset` com `redirectTo` igual a `${window.location.origin}/reset-password`; permanece em `/forgot-password` | Habilitado com o e-mail válido; desabilitado durante a submissão, com o rótulo `Enviando…` | Sucesso: "Se este e-mail tiver cadastro, você receberá um link para redefinir a senha." |
| `Redefinir senha` | `resetPassword` com o `token` da query; vai para `/login` | Habilitado com as duas senhas válidas e iguais; desabilitado durante a submissão, com o rótulo `Salvando…` | Sucesso: "Senha redefinida. Entre com a nova senha." · Erro: tabela da regra 5 |
| `Salvar` | `changePassword` com `revokeOtherSessions: true`; permanece na tela | Habilitado com os três campos válidos; desabilitado durante a submissão, com o rótulo `Salvando…` | Sucesso: "Senha alterada." · Erro: tabela da regra 5 |
| `Sair` | `signOut`, invalida o cache do Query; vai para `/login` | Sempre habilitado | Erro: "Não foi possível completar a ação. Tente de novo." |
| `Esqueci minha senha` | Link para `/forgot-password` | Sempre habilitado | — |
| `Criar conta` (link) | Link para `/signup` | Sempre habilitado | — |
| `Já tenho conta` | Link para `/login` | Sempre habilitado | — |
| `Trocar senha` | Link para `/app/account/password` | Sempre habilitado | — |
| `Pedir um novo link` | Link para `/forgot-password` | Visível só no estado de token inválido | — |
| `Voltar` / `Voltar para entrar` | Link para `/app` e para `/login`, respectivamente | Sempre habilitado | — |

---

## Fora de Escopo

- **Tema visual, tipografia, design system, ícones e cores do PWA** — pertencem à fase de design
  system, que ainda não tem issue. As telas desta spec usam utilitárias do Tailwind diretamente.
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

A numeração continua a da spec irmã, `docs/specs/003-autenticacao-api.md`, que fica com as tasks 1 a 7.

| # | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- |
| 8 | Create the sign-in and sign-up screens on apps/web | `@tanstack/react-query`, `@tanstack/react-form` e `@testing-library/*` instalados, `setupFilesAfterEnv` no `jest.config.json`, `QueryClientProvider` e o contexto do router em `main.tsx`, `shared/auth/`, `shared/http/`, `features/auth/api/` com schemas Zod e `messages.ts`, `features/auth/password-policy.ts`, rotas `/login`, `/signup` e `/verify-email` | Cenários 1, 2, 3, 4, 5, 11, 13, 16, 17, 18 e 19 verdes; os quatro gates do web saem com código 0 | 2, 3 |
| 9 | Protect the application area and sign out | `routes/(app)/route.tsx` com a guarda, `pendingComponent` e `errorComponent`, rota `/app`, ação `Sair`, `VITE_API_URL` no job `web` do CI | Cenários 7, 8, 9 e 12 verdes | 8 |
| 10 | Recover, reset and change the password on apps/web | Rotas `/forgot-password`, `/reset-password` e `/app/account/password` | Cenários 10, 14 e 15 verdes | 3, 6, 9 |
| 11 | Make apps/web an installable PWA | `vite-plugin-pwa` em `vite.config.ts`, manifesto da regra 9, ícones em `apps/web/public/` | Cenários 20 e 21 verdes | — |

A task 11 não depende de outra task desta spec, mas só fecha quando a fase de design system entregar
os três ícones da regra 9.

O cenário 6, de largura de 390px, é verificado nas tasks 8, 9 e 10, sobre as telas que cada uma
entrega.

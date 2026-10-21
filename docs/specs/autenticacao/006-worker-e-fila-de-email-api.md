# 006 — Enviar o e-mail por fila e purgar o que venceu num worker (API)

> **Status:** publicada
> **Perfil:** API
> **Módulo:** `apps/api`: `crates/worker/` novo, `crates/core/`, `crates/app/` e `migrations/`
> **Epic:** #3 — Autenticação e sessão com JWT, refresh e Google
> **Issues:** #71 e #161
> **Requisitos:** FR-ACC-07, FR-ACC-10, FR-ACC-15
> **Sprint:** 2
> **Decisão:** `docs/decisions/0017-fila-de-email-no-postgres-e-worker-com-cron.md`
> **Substitui na 003:** a regra 17, o Cenário 34, a última linha do Cenário 29, e da regra 13 e do
> Cenário 32 só o que diz como o e-mail sai

## Acceptance Criteria

### Contrato

Nenhuma rota muda. O contrato desta spec é o de um processo:

| Processo | Binário | Entrada | Lê | Escreve | Agenda |
| --- | --- | --- | --- | --- | --- |
| worker | `worker`, do pacote `clinicore-worker` | `cargo run -p clinicore-worker`, sob `infisical run --path=/api --` | `email_outbox`, `sessions`, `verifications`, `email_dispatches` | as mesmas, e o SMTP | a fila a cada 5 segundos; a purga às 03:00 de `America/Sao_Paulo` |

`POST /users`, `POST /email-verifications`, `POST /sessions` (que reenvia o link a quem entra sem ter
confirmado) e `POST /password-resets` respondem o que respondem hoje. A única diferença observável é que o e-mail
passa a sair do worker, até 5 segundos depois da resposta.

### Request

N/A — o worker não recebe requisição. A entrada dele é a linha da `email_outbox`, descrita no Data model
impact.

### Response

N/A — o worker não responde a ninguém. O que ele produz é o e-mail, a linha apagada ou atualizada, e o
log das regras 4 a 7.

### Perfis e privilégios

N/A — o worker é um processo do servidor, sem usuário. Quem pede o e-mail é a rota que já existe, com a
autorização dela.

---

## Regras de Negócio

### 1. O worker é um processo próprio, e lê só a configuração que usa

- `crates/worker/` gera o binário `worker`, que depende do `clinicore-core` e não depende do
  `clinicore-app`.
- **Lê oito variáveis, todas obrigatórias:** `APP_ENV`, `LOG_LEVEL`, `DATABASE_URL`,
  `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD` e `MAIL_FROM`, com os formatos que o
  `crates/core/src/config.rs` já usa. A leitura e as mensagens são as do `Config` da API, num
  `WorkerConfig` ao lado dele.
- **Faltando ou inválida qualquer uma**, o worker escreve em `stderr` a linha `Invalid environment:`,
  uma linha por variável recusada, em ordem alfabética, e sai com código `1` antes de abrir conexão.
- **A API deixa de exigir `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD` e `MAIL_FROM`**, porque
  não fala mais com o SMTP. Elas saem do `Config` da API e continuam no `.env.example`, agora lidas só
  pelo worker.
- O log é o mesmo subscriber do `tracing` da API, JSON em `stdout` fora de `development`. O
  `telemetry.rs` desce do `crates/app/` para o `crates/core/`.

### 2. A purga roda uma vez por dia, às 03:00 de São Paulo

- O worker calcula a próxima execução da expressão `0 3 * * *` no fuso `America/Sao_Paulo`, dorme até
  ela, executa, e calcula a seguinte. A expressão é lida pelo `croner` 4.0.1 sobre o `jiff` 0.2.38,
  com o banco de fusos embutido (`tzdb-bundle-always`).
- **Ao subir, o worker não purga.** A primeira execução é a próxima 03:00.
- **Duas instâncias do worker purgam duas vezes**, e a segunda apaga zero linhas. Não há lock.
- A API deixa de purgar: o `tokio::spawn(purge::run)` sai do `crates/app/src/main.rs`, e o
  `crates/app/src/purge.rs` vai para `crates/worker/src/purge.rs`.

### 3. A purga apaga o que venceu e nada mais

| Tabela | Condição |
| --- | --- |
| `sessions` | `expires_at` até agora, ou `created_at` até agora menos 30 dias |
| `verifications` | `expires_at` até agora |
| `email_dispatches` | `created_at` até agora menos 24 horas |
| `email_outbox` | `expires_at` até agora |

- Os 30 dias e as 24 horas são `SESSION_ABSOLUTE_LIFETIME` e `DISPATCH_WINDOW`, que descem do
  `crates/app/` para `crates/core/src/lifetimes.rs`, porque a API e o worker usam os dois.
- **Sessão apagada não vai para a denylist.** O access token dela já expirou, e o refresh é conferido
  contra a tabela.
- Cada execução registra em `info` a mensagem `purged the expired authentication records`, com os
  campos `sessions`, `verifications`, `email_dispatches` e `email_outbox`, cada um com a quantidade
  apagada.
- Uma falha registra em `error` a mensagem `could not purge the expired authentication records`, com o
  campo `error`, e o worker segue para a próxima 03:00.

### 4. A API grava o e-mail na fila, na mesma transação do token

- O cadastro, o pedido de confirmação, o reenvio e o pedido de redefinição renderizam o e-mail como
  hoje (assunto e corpos da regra 13 da 003, sem mudança de texto) e gravam uma linha em `email_outbox`
  **na transação que grava a `verifications`**. Se a transação desfaz, não há token nem e-mail.
- A linha nasce `pending`, com `attempts = 0`, `next_attempt_at = now()` e o `expires_at` igual ao da
  `verifications` gravada junto.
- **Nenhum caminho que hoje não envia passa a gravar.** Cadastro repetido, endereço sem conta, conta já
  confirmada e pedido barrado pelo limite da regra 15 da 003 não gravam linha.
- A API não abre conexão SMTP. O `Mailer` sai do `AppState` e vai para o worker; o `compose` e os
  templates `askama` ficam no `crates/core/`.

### 5. O worker reserva, envia e apaga

- A cada 5 segundos o worker reserva até 10 linhas `pending` com `next_attempt_at` até agora e
  `expires_at` depois de agora, na ordem de `next_attempt_at`, com `FOR UPDATE SKIP LOCKED`.
- **A reserva soma 1 a `attempts` e empurra `next_attempt_at` 2 minutos**, e é comitada antes do envio.
  Assim nenhuma transação fica aberta durante o SMTP, e uma linha reservada por um worker que caiu volta
  a ser elegível quando os 2 minutos passam.
- O worker envia as linhas reservadas uma depois da outra, pelo `lettre` com a feature `pool` e
  `timeout` de 10 segundos por envio.
- **Enviada, a linha é apagada.** O envio não grava log.
- Encontrando a fila vazia, o worker espera os 5 segundos seguintes. Uma falha da consulta de reserva
  registra em `error` a mensagem `could not claim the pending mail`, com o campo `error`, e o worker
  tenta de novo em 5 segundos.

### 6. O envio que falha é tentado de novo, com backoff, até 5 vezes

| Falha número | Próxima tentativa |
| --- | --- |
| 1 | 1 minuto depois |
| 2 | 2 minutos depois |
| 3 | 4 minutos depois |
| 4 | 8 minutos depois |
| 5 | nenhuma |

- Da 1ª à 4ª falha, o worker grava o novo `next_attempt_at` e registra em `warn` a mensagem
  `mail delivery failed, retrying`, com os campos `email` (mascarado como hoje: `a***@exemplo.com`),
  `attempt` e `reason`.
- **Na 5ª falha**, a linha passa a `failed`, `html_body` e `text_body` passam a `NULL` na mesma escrita,
  e o worker registra em `error` a mensagem `mail delivery gave up`, com `email` mascarado, `attempts` e
  `reason`.
- **Linha cujo `expires_at` passou não é enviada**, mesmo `pending`: o link dela já não abre nada. Ela
  espera a purga.
- O SMTP fora do ar não muda a resposta da API, que já respondeu `202` antes de o worker tentar.

### 7. O worker para sem perder o e-mail em voo

- Com `SIGTERM` ou `Ctrl+C`, o worker para de reservar, termina o envio em andamento, grava o resultado
  dele, e sai com código `0`. As linhas reservadas que não chegaram a ser enviadas voltam à fila quando
  a reserva vence.
- A purga em andamento termina antes da saída.

### 8. Persistência e auditoria

- **Tabelas e colunas alteradas:** `email_outbox`, nova, e o tipo `email_outbox_status`, novo. As
  outras quatro tabelas só perdem linhas, como já perdem hoje.
- **Auditoria:** N/A — e-mail transacional e purga não são operação de usuário sobre dado clínico. O
  rastro é o log das regras 3, 5 e 6.
- **Eventos e integrações:** o SMTP do Gmail, como na regra 13 da 003, agora só pelo worker.

---

## Erros

N/A — nenhuma rota nova, e nenhuma rota existente ganha ou perde código de erro. As falhas do worker
saem no log, com as mensagens literais das regras 1, 3, 5 e 6.

## Efeitos Colaterais

- **Persistência:** a API grava `email_outbox` `pending`; o worker apaga a enviada, atualiza a que falhou
  e marca `failed` a que esgotou; a purga apaga as quatro tabelas da regra 3.
- **Concorrência:** dois workers nunca reservam a mesma linha ao mesmo tempo, pelo `SKIP LOCKED`. Um
  worker que reservou e caiu devolve a linha em 2 minutos, e o e-mail pode chegar duas vezes, com o
  mesmo link.
- **Transação:** a linha da `email_outbox` é atômica com a `verifications` e, no cadastro, com o
  `users`, o `accounts` e o `email_dispatches`. A reserva do worker é uma transação curta, comitada antes
  do SMTP, e o resultado do envio é outra.

---

## Cenários de Aceite (Gherkin)

### Cenário 1 — A purga apaga o que venceu e nada mais, e repetir não apaga de novo (caminho feliz, regra 3)

```gherkin
Dado uma `sessions` vencida, uma passada dos 30 dias e uma válida
E uma `verifications` vencida e uma válida
E uma `email_dispatches` de 25 horas atrás e uma de 23 horas atrás
E uma `email_outbox` `failed` com `expires_at` de 1 segundo atrás e uma `pending` com `expires_at` daqui a 1 hora
Quando a purga é executada
Então ela devolve 2 sessões, 1 verificação, 1 envio e 1 linha da fila apagados
E as linhas válidas continuam existindo
E executar a purga de novo apaga 0 linhas de cada tabela
```

### Cenário 2 — A próxima purga é sempre a próxima 03:00 de São Paulo (caminho feliz, regra 2)

```gherkin
Dado o agendamento `0 3 * * *` no fuso `America/Sao_Paulo`
Quando a próxima execução é calculada a partir de `2026-10-20T02:59:59-03:00`
Então ela é `2026-10-20T03:00:00-03:00`
E a partir de `2026-10-20T03:00:00-03:00` ela é `2026-10-21T03:00:00-03:00`
```

### Cenário 3 — O worker com o ambiente incompleto não sobe (exceção, regra 1)

```gherkin
Dado o ambiente do worker sem `DATABASE_URL` e sem `SMTP_PASSWORD`
Quando o worker é inicializado
Então o processo escreve em `stderr` a linha `Invalid environment:`
E escreve a linha `  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)`
E escreve a linha `  SMTP_PASSWORD: expected a non-empty string`
E sai com código `1`
```

### Cenário 4 — A API sobe sem as variáveis do SMTP (caminho alternativo, regra 1)

```gherkin
Dado o ambiente completo da API sem `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD` e `MAIL_FROM`
Quando a configuração da API é lida
Então ela é aceita
```

### Cenário 5 — O cadastro grava o e-mail na fila, com o token (caminho feliz, regra 4)

```gherkin
Dado que o e-mail `ana@exemplo.com` não tem conta
Quando é enviado `POST /users` com nome, esse e-mail e a senha `Clinica#2026`
Então o sistema responde `202` com corpo vazio
E existe uma linha `pending` em `email_outbox` para `ana@exemplo.com`, com `attempts` 0
E o assunto dela é "Confirme seu e-mail no Clinicore"
E o token do link no corpo dela confirma o e-mail em `POST /email-verifications/confirmation`
E o `expires_at` dela é igual ao da `verifications` desse endereço
E nenhuma conexão SMTP foi aberta pela API
```

### Cenário 6 — Pedido de confirmação e de redefinição também vão para a fila (caminho alternativo, regra 4)

```gherkin
Dado uma conta não confirmada de `ana@exemplo.com`
Quando é enviado `POST /email-verifications` para esse endereço
Então existe uma linha `pending` em `email_outbox` com o assunto "Confirme seu e-mail no Clinicore"
Dado uma conta confirmada de `bia@exemplo.com`
Quando é enviado `POST /password-resets` para esse endereço
Então existe uma linha `pending` em `email_outbox` com o assunto "Redefinir sua senha do Clinicore"
```

### Cenário 7 — O que hoje não envia não grava na fila (exceção, regra 4)

```gherkin
Dado uma conta confirmada de `ana@exemplo.com`
Quando é enviado `POST /users` com esse e-mail
E é enviado `POST /email-verifications` para esse e-mail
E é enviado `POST /password-resets` para `ninguem@exemplo.com`, sem conta
Então as três respostas são `202` com corpo vazio
E não existe linha em `email_outbox`
```

### Cenário 8 — O worker envia e apaga a linha (caminho feliz, regra 5)

```gherkin
Dado uma linha `pending` em `email_outbox` para `ana@exemplo.com`, com `next_attempt_at` agora e `expires_at` daqui a 1 hora
Quando o worker processa a fila com o SMTP respondendo
Então o SMTP recebe um e-mail para `ana@exemplo.com` com o assunto e os dois corpos da linha
E a linha deixa de existir
```

### Cenário 9 — O SMTP falha, e a linha volta com backoff (exceção, regra 6)

```gherkin
Dado uma linha `pending` em `email_outbox` para `ana@exemplo.com`, com `attempts` 0
Quando o worker processa a fila com o SMTP recusando o envio
Então a linha continua `pending`, com `attempts` 1 e `next_attempt_at` 1 minuto depois da falha
E os dois corpos continuam gravados
E o log tem uma linha `warn` com a mensagem `mail delivery failed, retrying`, `email` `a***@exemplo.com`, `attempt` 1 e o motivo
```

### Cenário 10 — A quinta falha desiste e apaga o corpo (exceção, regra 6)

```gherkin
Dado uma linha `pending` em `email_outbox` para `ana@exemplo.com`, com `attempts` 4
Quando o worker processa a fila com o SMTP recusando o envio
Então a linha passa a `failed`, com `attempts` 5
E `html_body` e `text_body` são `NULL`
E o log tem uma linha `error` com a mensagem `mail delivery gave up`, `email` `a***@exemplo.com`, `attempts` 5 e o motivo
E o worker não reserva mais essa linha
```

### Cenário 11 — Linha vencida não é enviada (exceção, regra 6)

```gherkin
Dado uma linha `pending` em `email_outbox` com `expires_at` de 1 segundo atrás
Quando o worker processa a fila
Então o SMTP não recebe nenhum e-mail
E a linha continua `pending`, com `attempts` 0
```

### Cenário 12 — Dois workers não reservam a mesma linha, e a reserva que ninguém fechou volta (caminho alternativo, regra 5)

```gherkin
Dado 10 linhas `pending` em `email_outbox`, elegíveis
Quando duas reservas de até 10 linhas rodam ao mesmo tempo
Então cada linha está em exatamente uma das duas reservas
E cada linha reservada tem `attempts` 1 e `next_attempt_at` 2 minutos depois da reserva
Dado uma linha reservada cujo `next_attempt_at` passou sem resultado gravado
Quando o worker reserva de novo
Então essa linha volta a ser reservada, com `attempts` 2
```

### Cenário 13 — O worker desligado termina o envio em andamento (caminho alternativo, regra 7)

```gherkin
Dado o worker enviando a linha de `ana@exemplo.com`, com o SMTP levando 1 segundo para responder
Quando o worker recebe o sinal de parada durante o envio
Então o SMTP recebe o e-mail
E a linha deixa de existir
E o worker não reserva nenhuma outra linha
E o processo termina com código `0`
```

---

## Architecture impact

- **Blocos de construção:** o bloco `worker` passa de "M1 · #71, sem arquivo até existir" a existente,
  com `building-blocks/worker.md`. O `apps/api` deixa de falar com o SMTP. O componente `Purga` sai do
  `api.md` e o `Telemetria` muda de crate. O `components-api.drawio.svg` e o `containers.drawio.svg`
  acompanham.
- **Runtime:** `runtime-sign-up-and-email-verification.drawio.svg` passa a mostrar o e-mail gravado na
  fila e enviado pelo worker; `session-lifecycle.md` diz que a purga já roda no worker, às 03:00.
- **Deployment:** a linha `worker` do `deployment.md` deixa de dizer "quando a #71 o criar". A aplicação
  do Coolify continua com a #4.
- **Configuração:** `concepts/configuration.md` ganha a linha do worker, com o mesmo comportamento de
  boot.
- **Riscos:** sai "A purga roda dentro de cada réplica da API".
- **Repo:** a linha "Fila e agendamento" e a linha "E-mail" da tabela de stack do `AGENTS.md` da raiz,
  e a árvore de arquitetura dele, ganham o `crates/worker/`; o `apps/api/AGENTS.md` atualiza a seção
  Crates e a Estrutura.

## Data model impact

- **Entidade nova `EmailOutbox`:** um e-mail transacional esperando o envio. Liga-se ao `User` pelo
  endereço, como a `Verification` e o `EmailDispatch`.
- **Tabela `email_outbox`:**

  | Coluna | Tipo | Nulo | Origem |
  | --- | --- | --- | --- |
  | `id` | `uuid`, `DEFAULT gen_random_uuid()` | não | o banco |
  | `email` | `text`, `CHECK (email = lower(email))` | não | o endereço da rota, já em minúsculas |
  | `subject` | `text` | não | o assunto da regra 13 da 003 |
  | `html_body` | `text` | só quando `failed` | o template `mail.html` renderizado |
  | `text_body` | `text` | só quando `failed` | o template `mail.txt` renderizado |
  | `status` | `email_outbox_status` (`pending`, `failed`), `DEFAULT 'pending'` | não | a API grava `pending`; o worker grava `failed` |
  | `attempts` | `integer`, `DEFAULT 0`, `CHECK (attempts BETWEEN 0 AND 5)` | não | o worker, na reserva |
  | `next_attempt_at` | `timestamptz`, `DEFAULT now()` | não | o worker, na reserva e na falha |
  | `expires_at` | `timestamptz` | não | o `expires_at` da `verifications` da mesma transação |
  | `created_at` | `timestamptz`, `DEFAULT now()` | não | o banco |

- **Invariante:** `status = 'pending'` se e somente se `html_body` e `text_body` não são nulos, por
  `CHECK`.
- **Índices:** `email_outbox (next_attempt_at) WHERE status = 'pending'`, para a reserva, e
  `email_outbox (expires_at)`, para a purga.
- **Migration:** a segunda do repo, `apps/api/migrations/<timestamp>_email_outbox.sql`.
- **Padrões de acesso:** "Cadastro", "Pedir novo link" e "Pedir redefinição de senha" passam a escrever
  `email_outbox`; entram "Reservar e-mail", "Resultado do envio" e a "Purga" passa a ser diária e a
  incluir `email_outbox`.

## Fora de Escopo

- **O Dockerfile, a imagem e a aplicação do worker no Coolify.** Nem a API tem Dockerfile; os dois
  nascem com a #4, como diz a ADR 0014.
- **Reenvio manual da linha `failed`.** A pessoa pede o link de novo pelo botão de reenvio, que já
  existe; o operador vê a desistência no log.
- **Envio em paralelo.** Uma mensagem por vez atende o volume da clínica piloto, e o teto da conta do
  Gmail da regra 13 da 003 é menor que a vazão de um worker.
- **Recuperar a purga perdida.** O worker parado às 03:00 purga na 03:00 seguinte.
- **A cota contar só o e-mail que sai.** É a #136, que vem depois desta e passa a contar sobre a
  `email_outbox`.
- **O semáforo e o `TaskTracker` da #132.** Deixam de ter onde morar quando a API para de enviar; o
  `pool` e o `timeout` da #132 entram na regra 5, e a #132 fecha com esta spec.
- **`LISTEN`/`NOTIFY` para acordar o worker.** O polling de 5 segundos cabe no tempo que a pessoa leva
  para abrir o e-mail.

## Quebra em Tasks

| # | Issue | Título | Escopo | Critério de aceite | Depende de |
| --- | --- | --- | --- | --- | --- |
| 1 | #71 | Purge expired authentication records daily in a worker | `crates/worker/` novo com `main.rs` e `purge.rs`; `purge.rs` e o `spawn` saem do `crates/app/`; `WorkerConfig` só com `APP_ENV`, `LOG_LEVEL` e `DATABASE_URL`; `telemetry.rs` e `lifetimes.rs` no `crates/core/`; o teste da purga vai para `crates/worker/tests/`; ADR 0017 e os docs de arquitetura do worker. Consumidores do que muda de lugar: `purge.rs` (1 teste, `main.rs`), `telemetry.rs` (`main.rs` e `tests/api/support.rs`), as duas constantes (`email_dispatches.rs`, `sessions/tokens/refresh.rs` e `sessions/queries.rs`) | Cenários 1 (sem a `email_outbox`), 2 e 3 (com `DATABASE_URL`) | — |
| 2 | #161 | Send transactional email through a Postgres outbox in the worker | migration da `email_outbox`; a gravação nas consultas do cadastro, da confirmação e da redefinição; `Mailer` do `crates/core/` para o worker, com `pool` e `timeout`; o `AppState` e o `Config` da API sem SMTP; a fila no worker; `.sqlx/`; os testes que hoje leem o `AsyncStubTransport` (`users/sign_up.rs`, `sessions/sign_in.rs`, `sessions/current_session.rs`, `password_resets/request.rs`, `support.rs`) passam a ler a `email_outbox`; docs de dados e de runtime. Começa depois da #133, que junta a reserva e o token numa transação. Fecha a #132 | Cenários 1 (com a `email_outbox`), 3 (com `SMTP_PASSWORD`), 4 a 13 | 1 |

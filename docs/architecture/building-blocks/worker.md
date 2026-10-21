# worker

O processo da API que trabalha fora da requisição, em Rust, no `apps/api/crates/worker/`. Hoje ele
purga o que venceu, uma vez por dia. A fila de e-mail (`email_outbox`) chega com a #161
([spec 006](../../specs/autenticacao/006-worker-e-fila-de-email-api.md),
[ADR 0017](../../decisions/0017-fila-de-email-no-postgres-e-worker-com-cron.md)).

**Nunca faz:** responder HTTP, depender do `clinicore-app`, guardar estado em memória que outra
instância precise ver, purgar no boot.

## Blocos internos

![Componentes do worker](../diagrams/components-worker.drawio.svg)

| Bloco | Onde | Responsabilidade |
| --- | --- | --- |
| Boot | `crates/worker/src/main.rs` | lê o `WorkerConfig`, liga o `tracing`, abre o pool e entrega o pool ao agendamento |
| Agendamento | `crates/worker/src/schedule.rs` | calcula a próxima `0 3 * * *` em `America/Sao_Paulo` com o `croner` sobre o `jiff`, dorme até ela, purga e repete |
| Purga | `crates/worker/src/purge.rs` | apaga a sessão vencida ou que passou dos 30 dias, a verificação vencida e o registro de envio com mais de 24 horas, e registra as quantidades no log |
| `core` | `crates/core/` | configuração, pool do Postgres, subscriber do `tracing` e os prazos `SESSION_ABSOLUTE_LIFETIME` e `DISPATCH_WINDOW`, compartilhados com a API |

## O que ele garante

- **A primeira purga é a próxima 03:00.** Se o worker estiver parado às 03:00, a purga daquele dia não
  acontece; ela só volta na 03:00 seguinte.
- **Duas instâncias purgam duas vezes, e a segunda apaga zero linhas.** Não há lock: o `DELETE` é
  idempotente.
- **Uma purga que falha é registrada em `error`** e espera a 03:00 seguinte.
- **O ambiente incompleto impede o boot:** o worker escreve `Invalid environment:` e uma linha por
  variável recusada, e sai com código 1 antes de abrir conexão.

## Para mudar com segurança

Rodar: `infisical run --path=/api -- cargo run -p clinicore-worker`. Os testes ficam em
`crates/worker/tests/worker/`, com as mesmas regras de teste da API, descritas no
[`apps/api/AGENTS.md`](https://github.com/TheoOdawara/clinicore/blob/staging/apps/api/AGENTS.md). O
horário e o fuso são o comportamento: trocar o cron por um intervalo muda o que a spec 006 decidiu.

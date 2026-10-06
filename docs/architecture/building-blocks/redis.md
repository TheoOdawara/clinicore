# Redis 8

O armazenamento de vida curta da API. Guarda duas coisas e nada mais.

| O que | Chave | Vida |
| --- | --- | --- |
| Contagem do limite de requisições | por IP, por sessão, por e-mail e por faixa de rede, conforme a rota | a janela do limite |
| Denylist de revogação de sessão | `auth:revoked:<sessão>` | 7 dias |

**Nunca guarda:** dado de negócio, cache de consulta, sessão. A sessão mora na tabela `sessions` do
PostgreSQL.

## O que ele exige

- **Persistência.** O Redis sobe com `--appendonly yes` em todo ambiente. Um Redis que reinicia vazio
  devolve a validade a access tokens de sessões já apagadas
  ([ADR 0012](../../decisions/0012-revogacao-de-sessao-antes-de-apagar.md)).
- **Disponibilidade para as rotas limitadas.** Com o Redis fora, a rota limitada responde `503` e
  nenhuma sessão é apagada; o `/health` continua `200`.

## Para mudar com segurança

As regras de limite em duas chaves e a ordem da revogação estão no
[`apps/api/AGENTS.md`](../../../apps/api/AGENTS.md), em Regras.

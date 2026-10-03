# Segurança — `SEC`

O isolamento entre clientes e a proteção do dado de saúde, que a LGPD trata como sensível.

## NFR-SEC-01 — Isolamento entre clientes

O sistema deve impedir que um usuário de um cliente leia ou altere dado de outro cliente.

| Atributo | Valor |
| --- | --- |
| Justificativa | Cada cliente é isolado dos demais. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-SEC-01.1** — Dado um usuário da clínica A, quando busca pacientes, então a resposta traz 0 registros da clínica B.

## NFR-SEC-02 — Criptografia em trânsito e em repouso

O sistema deve trafegar todo dado por conexão cifrada e armazenar cifrado todo dado de saúde.

| Atributo | Valor |
| --- | --- |
| Justificativa | Dado de saúde é dado sensível na LGPD. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-SEC-02.1** — 100% das conexões entre cliente e servidor usam TLS.
- **NFR-SEC-02.2** — 100% dos volumes e backups que guardam dado de saúde estão cifrados.

## NFR-SEC-03 — Menor privilégio por papel

O sistema deve negar toda ação que o papel do usuário não autoriza explicitamente.

| Atributo | Valor |
| --- | --- |
| Justificativa | O acesso padrão a dado de saúde é nenhum. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-SEC-03.1** — Para cada papel, 100% das ações não listadas como permitidas a ele são recusadas.

## NFR-SEC-04 — Dado de saúde no aparelho

O sistema deve manter cifrado o dado de saúde guardado no app e apagá-lo após 72 horas sem sincronizar, no logout e na revogação da sessão, preservando a fila de escrita pendente.

| Atributo | Valor |
| --- | --- |
| Justificativa | Aparelho perdido não pode expor prontuário; o que ainda não subiu não pode se perder. |
| Origem | [ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-SEC-04.1** — Dado um app sem sincronizar há 72 horas, quando é aberto, então mostra 0 dados de saúde e mantém a fila pendente.
- **NFR-SEC-04.2** — Dado um logout, quando ele conclui, então o aparelho guarda 0 dados de saúde.

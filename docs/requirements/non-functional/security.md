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

## NFR-SEC-02 — Criptografia em trânsito

O sistema deve trafegar todo dado por conexão cifrada.

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

## NFR-SEC-03 — Menor privilégio por papel

O sistema deve negar toda ação que o papel do usuário não autoriza explicitamente.

| Atributo | Valor |
| --- | --- |
| Justificativa | O acesso padrão a dado de saúde é nenhum. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | retired |
| Aposentado | v2.0.0: duplicava o [FR-ACC-03](../functional/access.md#fr-acc-03--papéis-com-permissões-distintas), que recebeu o critério dele como FR-ACC-03.2 |
| Marco | — |
| Desde | v1.0.0 |

**Critérios de aceite**

- **NFR-SEC-03.1** — Para cada papel, 100% das ações não listadas como permitidas a ele são recusadas.

## NFR-SEC-04 — Dado de saúde no aparelho

O sistema deve manter cifrado o dado de saúde guardado no app.

| Atributo | Valor |
| --- | --- |
| Justificativa | Aparelho perdido não pode expor prontuário. |
| Origem | [ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**

- **NFR-SEC-04.1** — 100% do dado de saúde guardado pelo app no aparelho está cifrado.

## NFR-SEC-05 — Criptografia em repouso

O sistema deve armazenar cifrado todo dado de saúde.

| Atributo | Valor |
| --- | --- |
| Justificativa | Dado de saúde é dado sensível na LGPD. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v2.0.0 |

**Critérios de aceite**

- **NFR-SEC-05.1** — 100% dos volumes e backups que guardam dado de saúde estão cifrados.

## NFR-SEC-06 — Dado de saúde apagado do aparelho

O sistema deve apagar do app o dado de saúde após 72 horas sem sincronizar, no logout e na revogação da sessão, preservando a fila de escrita pendente.

| Atributo | Valor |
| --- | --- |
| Justificativa | Aparelho perdido não pode expor prontuário; o que ainda não subiu não pode se perder. |
| Origem | [ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v2.0.0 |

**Critérios de aceite**

- **NFR-SEC-06.1** — Dado um app sem sincronizar há 72 horas, quando é aberto, então mostra 0 dados de saúde e mantém a fila pendente.
- **NFR-SEC-06.2** — Dado um logout, quando ele conclui, então o aparelho guarda 0 dados de saúde.

## NFR-SEC-07 — Nenhuma resposta da API no cache do navegador

O sistema deve impedir que o navegador guarde em cache qualquer resposta da API.

| Atributo | Valor |
| --- | --- |
| Justificativa | Um dispositivo compartilhado da clínica não pode reter dado de saúde. |
| Origem | Theo, ao especificar a autenticação: [spec 003 — web](../../specs/autenticacao/003-autenticacao-web.md), regra 9 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v2.0.0 |

**Critérios de aceite**

- **NFR-SEC-07.1** — Depois de usar o web, o cache do navegador contém 0 URLs da origem da API.

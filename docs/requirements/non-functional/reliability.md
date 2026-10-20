# Confiabilidade — `REL`

O quanto o sistema fica disponível e o que acontece quando a conexão falha.

## NFR-REL-01 — Disponibilidade

O sistema deve manter a API e o web disponíveis em 99,5% do tempo de cada mês.

| Atributo | Valor |
| --- | --- |
| Justificativa | Agenda e prontuário são críticos na operação diária da clínica. |
| Origem | Theo, elicitação de 2026-10-03 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**

- **NFR-REL-01.1** — A indisponibilidade somada da API e do web em produção não passa de 0,5% dos minutos do mês, cerca de 3h39.

## NFR-REL-02 — Consulta offline no app

O sistema deve apresentar, no app sem conexão, os dados já sincronizados.

| Atributo | Valor |
| --- | --- |
| Justificativa | O dentista atende em consultório com sinal ruim e não pode parar de consultar a agenda e o paciente. |
| Origem | [ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**

- **NFR-REL-02.1** — Dado um dentista com o app sincronizado, quando perde a conexão, então vê 100% das consultas sincronizadas da agenda dele e os pacientes delas.

## NFR-REL-03 — Conflito offline nunca sobrescreve

O sistema deve recusar a escrita feita offline sobre um dado que mudou no servidor e mantê-la em pendência para a pessoa refazer.

| Atributo | Valor |
| --- | --- |
| Justificativa | Sobrescrever em silêncio um dado de saúde é pior que recusar. |
| Origem | [ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**

- **NFR-REL-03.1** — Dada uma consulta cancelada na recepção enquanto o dentista estava offline, quando o registro dele sobe, então o sistema o recusa e o app o mostra em pendência: 0 sobrescritas silenciosas.

## NFR-REL-04 — Publicação sem indisponibilidade

O sistema deve publicar uma nova versão da API e do web sem recusar requisição durante a publicação.

| Atributo | Valor |
| --- | --- |
| Justificativa | A clínica opera o dia inteiro; uma publicação não pode derrubar a agenda nem o atendimento. |
| Origem | Theo, revisão de 2026-10-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v2.0.0 |

**Critérios de aceite**

- **NFR-REL-04.1** — Durante a publicação de uma nova versão, 0 requisições à API e ao web são recusadas por indisponibilidade.

## NFR-REL-05 — Gravação offline no app

O sistema deve gravar no app sem conexão os novos registros e enviá-los ao servidor quando a conexão volta.

| Atributo | Valor |
| --- | --- |
| Justificativa | O dentista atende em consultório com sinal ruim e não pode parar de registrar. |
| Origem | [ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v2.0.0 |

**Critérios de aceite**

- **NFR-REL-05.1** — Dado um dentista sem conexão, quando registra um atendimento, então o registro fica salvo no app como pendente de envio.
- **NFR-REL-05.2** — Quando a conexão volta, 100% dos registros pendentes chegam ao servidor sem ação do usuário.

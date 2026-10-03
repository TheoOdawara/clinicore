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

## NFR-REL-02 — Operação offline no app

O sistema deve permitir, no app sem conexão, consultar os dados já sincronizados e gravar novos registros, enviados à API quando a conexão volta.

| Atributo | Valor |
| --- | --- |
| Justificativa | O dentista atende em consultório com sinal ruim e não pode parar de registrar. |
| Origem | [ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-REL-02.1** — Dado um dentista com o app sincronizado, quando perde a internet, então vê a agenda e os pacientes dela e registra o atendimento.
- **NFR-REL-02.2** — Quando a conexão volta, 100% dos registros da fila chegam à API sem ação do usuário.

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
- **NFR-REL-03.1** — Dada uma consulta cancelada na recepção enquanto o dentista estava offline, quando o registro dele sobe, então a API o recusa e o app o mostra em pendência: 0 sobrescritas silenciosas.

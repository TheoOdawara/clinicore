# IA no atendimento — `AIA`

O agente que conversa com o paciente pelo WhatsApp no lugar da recepção.

## FR-AIA-01 — Agente de agendamento no WhatsApp

> Como recepção, quero que um agente confirme, remarque e agende consultas pelo WhatsApp, para que o paciente seja atendido fora do horário da recepção.

O sistema deve conduzir pelo WhatsApp, por um agente de IA, a confirmação, a remarcação e o agendamento de consultas.

| Atributo | Valor |
| --- | --- |
| Justificativa | O paciente é atendido sem ocupar a recepção. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-AIA-01.1** — Dado um paciente que pede remarcação no WhatsApp, quando há horário livre, então o agente remarca e a agenda é atualizada.

## FR-AIA-02 — Reativação de pacientes e de orçamentos

> Como gestor, quero que o agente procure pacientes inativos e orçamentos não aprovados, para que a clínica recupere receita parada.

O sistema deve enviar pelo WhatsApp, por um agente de IA, mensagem de reativação a pacientes inativos e a pacientes com orçamento não aprovado.

| Atributo | Valor |
| --- | --- |
| Justificativa | Paciente inativo e orçamento parado são receita a recuperar. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-AIA-02.1** — Dado um paciente inativo, quando a reativação é disparada, então ele recebe a mensagem e a resposta fica no histórico de comunicação.

## FR-AIA-03 — Passagem para humano

> Como recepção, quero assumir a conversa quando o agente não resolve, para que o paciente não fique sem resposta.

O sistema deve transferir a conversa para a recepção quando o agente não conclui o pedido do paciente.

| Atributo | Valor |
| --- | --- |
| Justificativa | O paciente não pode ficar sem resposta quando o agente não resolve. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-AIA-03.1** — Dado um pedido que o agente não conclui, quando isso acontece, então a conversa aparece para a recepção com o histórico.

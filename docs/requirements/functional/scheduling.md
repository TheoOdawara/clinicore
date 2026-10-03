# Agenda — `SCH`

A marcação de consultas, a comunicação com o paciente e o que acontece quando o horário não é cumprido.

## FR-SCH-01 — Agenda por dentista

> Como recepção, quero marcar e consultar os horários de cada dentista, para que eu saiba quem atende quem e quando.

O sistema deve manter a agenda de consultas de cada dentista.

| Atributo | Valor |
| --- | --- |
| Justificativa | A agenda é a operação diária da clínica. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SCH-01.1** — Dada uma consulta marcada para um dentista, quando a agenda dele é aberta no dia, então a consulta aparece no horário marcado.
- **FR-SCH-01.2** — Dado um horário já ocupado de um dentista, quando outra consulta que não é encaixe é marcada nele, então o sistema recusa.

## FR-SCH-02 — Agenda por cadeira ou sala

> Como recepção, quero ver a ocupação de cada cadeira ou sala, para que duas consultas não disputem o mesmo espaço.

O sistema deve manter a agenda de cada cadeira ou sala da clínica.

| Atributo | Valor |
| --- | --- |
| Justificativa | Duas consultas não ocupam a mesma cadeira ao mesmo tempo. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SCH-02.1** — Dada uma cadeira ocupada num horário, quando outra consulta que não é encaixe é marcada nela no mesmo horário, então o sistema recusa.

## FR-SCH-03 — Confirmação por WhatsApp

> Como recepção, quero que o paciente confirme a consulta pelo WhatsApp, para que eu não ligue para cada paciente.

O sistema deve enviar ao paciente um pedido de confirmação por WhatsApp e registrar a resposta na consulta.

| Atributo | Valor |
| --- | --- |
| Justificativa | A confirmação automática tira da recepção o contato um a um. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SCH-03.1** — Dada uma consulta marcada, quando chega a janela de confirmação, então o paciente recebe a mensagem no WhatsApp.
- **FR-SCH-03.2** — Dada a resposta do paciente, quando ela chega, então a consulta passa a confirmada ou a cancelada na agenda.

## FR-SCH-04 — Lembrete por WhatsApp

> Como recepção, quero que o paciente seja lembrado da consulta, para que ele não falte por esquecimento.

O sistema deve enviar ao paciente um lembrete da consulta por WhatsApp antes do horário.

| Atributo | Valor |
| --- | --- |
| Justificativa | O lembrete evita a falta por esquecimento. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SCH-04.1** — Dada uma consulta confirmada, quando chega a janela de lembrete, então o paciente recebe o lembrete no WhatsApp.

## FR-SCH-05 — Registro de falta

> Como recepção, quero registrar que o paciente não compareceu, para que a falta entre no histórico e no indicador.

O sistema deve permitir registrar a falta do paciente numa consulta.

| Atributo | Valor |
| --- | --- |
| Justificativa | Sem a falta registrada não existe taxa de faltas. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SCH-05.1** — Dada uma consulta cujo paciente não veio, quando a recepção registra a falta, então ela aparece no histórico do paciente.

## FR-SCH-06 — Remarcação

> Como recepção, quero mover uma consulta para outro horário, para que o vínculo com o paciente e o tratamento se mantenha.

O sistema deve permitir remarcar uma consulta para outro horário, mantendo o registro do horário original.

| Atributo | Valor |
| --- | --- |
| Justificativa | Apagar e recriar a consulta perde o registro do horário original. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SCH-06.1** — Dada uma consulta remarcada, quando o histórico do paciente é aberto, então mostra o horário original e o novo.

## FR-SCH-07 — Encaixe

> Como recepção, quero encaixar um paciente fora dos horários livres, para que uma urgência seja atendida no mesmo dia.

O sistema deve permitir marcar uma consulta de encaixe num horário já ocupado, identificada como encaixe.

| Atributo | Valor |
| --- | --- |
| Justificativa | Uma urgência precisa de atendimento fora dos horários livres. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SCH-07.1** — Dado um horário ocupado, quando a recepção marca um encaixe nele, então a consulta é criada e aparece identificada como encaixe.

## FR-SCH-08 — Lista de espera

> Como recepção, quero manter os pacientes que querem um horário mais cedo, para que eu preencha uma desistência.

O sistema deve manter uma lista de espera de pacientes e apresentá-la quando um horário é liberado.

| Atributo | Valor |
| --- | --- |
| Justificativa | Um horário liberado pode ser oferecido a quem espera. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-SCH-08.1** — Dado um paciente na lista de espera, quando uma consulta é cancelada, então a recepção vê a lista ao lado do horário liberado.

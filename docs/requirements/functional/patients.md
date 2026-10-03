# Pacientes — `PAT`

O cadastro do paciente e a visão única de tudo que a clínica sabe dele.

## FR-PAT-01 — Cadastro de paciente

> Como recepção, quero cadastrar e atualizar os dados de um paciente, para que a clínica tenha um registro só por paciente.

O sistema deve manter o cadastro do paciente com dados de identificação e de contato.

| Atributo | Valor |
| --- | --- |
| Justificativa | O paciente é a entidade à qual agenda, prontuário e financeiro se ligam. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-PAT-01.1** — Dado um paciente cadastrado, quando a recepção o busca pelo nome, então o encontra com os dados informados.

## FR-PAT-02 — Responsável legal para menor

> Como recepção, quero vincular um responsável legal ao paciente menor de idade, para que os documentos sejam assinados por quem pode assinar.

O sistema deve exigir um responsável legal no cadastro do paciente menor de 18 anos.

| Atributo | Valor |
| --- | --- |
| Justificativa | O prontuário de um menor é assinado pelo responsável (BR-04). |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-PAT-02.1** — Dado um paciente com menos de 18 anos, quando o cadastro é salvo sem responsável legal, então o sistema recusa.

## FR-PAT-03 — Histórico único do paciente

> Como dentista, quero ver num só lugar o histórico clínico, estético, financeiro e de comunicação do paciente, para que eu não procure a mesma pessoa em quatro telas.

O sistema deve apresentar, a partir do paciente, o histórico clínico, o estético, o financeiro e o de comunicação dele.

| Atributo | Valor |
| --- | --- |
| Justificativa | O legado espalha o paciente entre ferramentas; a visão única é o argumento de produto. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-PAT-03.1** — Dado um paciente com evolução, procedimento estético, parcela e mensagem de confirmação registrados, quando o dentista abre o histórico, então vê os quatro.

# Portabilidade — `PRT`

A entrada dos dados que a clínica tinha em outro sistema e a saída de todos os dados dela.

## FR-PRT-01 — Importação do Simples Dental

> Como gestor, quero importar pacientes, agenda, prontuário e financeiro do Simples Dental, para que eu troque de sistema sem redigitar.

O sistema deve importar pacientes, agenda, prontuário e financeiro a partir de uma exportação do Simples Dental.

| Atributo | Valor |
| --- | --- |
| Justificativa | Aprisionamento de dados é o que impede a clínica de sair do legado. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-PRT-01.1** — Dada uma exportação do Simples Dental, quando é importada, então os pacientes, a agenda, o prontuário e o financeiro dela ficam disponíveis no sistema.

## FR-PRT-02 — Relatório de importação

> Como gestor, quero saber o que entrou, o que falhou e por quê, para que nada suma em silêncio.

O sistema deve produzir, a cada importação, um relatório com cada registro importado e cada registro recusado com o motivo.

| Atributo | Valor |
| --- | --- |
| Justificativa | A exportação do legado vem incompleta ou errada; a falha precisa ser visível. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-PRT-02.1** — Dada uma importação, quando ela termina, então todo registro da origem aparece como importado ou como recusado com o motivo.

## FR-PRT-03 — Exportação completa self-service

> Como gestor, quero exportar todos os dados da minha clínica quando eu quiser, para que eu não dependa do fornecedor para sair.

O sistema deve entregar ao gestor, a pedido dele e sem intervenção de suporte, todos os dados da clínica em formato aberto e documentado.

| Atributo | Valor |
| --- | --- |
| Justificativa | Devolver ao cliente o controle dos próprios dados é a razão de existir do produto. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-PRT-03.1** — Dado um gestor, quando pede a exportação, então recebe todos os dados da clínica em formato aberto e documentado, sem depender de suporte.

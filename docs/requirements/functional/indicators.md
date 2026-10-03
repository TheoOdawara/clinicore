# Indicadores de gestão — `KPI`

Os números que o gestor e o dono de rede acompanham.

## FR-KPI-01 — Funil de orçamento

> Como gestor, quero ver quantos orçamentos foram apresentados e quantos foram aceitos, para que eu saiba onde a venda se perde.

O sistema deve apresentar, por período, a quantidade e o valor de orçamentos apresentados e de orçamentos aceitos.

| Atributo | Valor |
| --- | --- |
| Justificativa | O funil mostra onde a venda se perde entre a proposta e o aceite. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-KPI-01.1** — Dado um mês com orçamentos apresentados e aceitos, quando o gestor abre o painel, então vê as duas quantidades e a taxa de aceite.

## FR-KPI-02 — Ocupação de cadeira e de dentista

> Como gestor, quero ver a ocupação e a ociosidade de cada cadeira e de cada dentista, para que eu ajuste a escala.

O sistema deve apresentar, por período, a taxa de ocupação de cada cadeira e de cada dentista.

| Atributo | Valor |
| --- | --- |
| Justificativa | Ocupação e ociosidade orientam a escala da clínica. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-KPI-02.1** — Dado um mês de agenda, quando o gestor abre o painel, então vê a taxa de ocupação por cadeira e por dentista.

## FR-KPI-03 — Taxa de faltas

> Como gestor, quero ver a proporção de consultas com falta, para que eu meça o efeito da confirmação.

O sistema deve apresentar, por período, a taxa de faltas sobre as consultas marcadas.

| Atributo | Valor |
| --- | --- |
| Justificativa | A taxa de faltas mede se a confirmação por WhatsApp funciona. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-KPI-03.1** — Dado um mês com 100 consultas marcadas e 8 faltas, quando o gestor abre o painel, então a taxa de faltas é 8%.

## FR-KPI-04 — Faturamento e inadimplência

> Como gestor, quero ver o faturamento e a inadimplência do período, para que eu acompanhe o resultado.

O sistema deve apresentar, por período, o valor faturado e o valor inadimplente.

| Atributo | Valor |
| --- | --- |
| Justificativa | Faturamento e inadimplência são acompanhados por período. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-KPI-04.1** — Dada uma parcela vencida sem pagamento, quando o gestor abre o painel, então o valor dela compõe a inadimplência do período.

## FR-KPI-05 — Pacientes inativos e reativados

> Como gestor, quero ver quantos pacientes estão inativos e quantos voltaram, para que eu meça a retenção.

O sistema deve apresentar, por período, a quantidade de pacientes inativos e a de pacientes reativados.

| Atributo | Valor |
| --- | --- |
| Justificativa | Inativos e reativados medem a retenção da clínica. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-KPI-05.1** — Dado um mês de operação, quando o gestor abre o painel, então vê a quantidade de inativos e a de reativados.

## FR-KPI-06 — Visão por clínica e consolidada por rede

> Como dono de rede, quero ver os indicadores de cada clínica e o consolidado da rede, para que eu compare as unidades.

O sistema deve apresentar cada indicador por clínica e, ao dono de rede, também consolidado.

| Atributo | Valor |
| --- | --- |
| Justificativa | O dono de rede decide olhando o conjunto e a diferença entre unidades. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-KPI-06.1** — Dado um dono de rede com 3 clínicas, quando abre os indicadores, então vê o consolidado e cada clínica.
- **FR-KPI-06.2** — Dado o gestor de uma delas, quando abre os indicadores, então vê só os da clínica dele.

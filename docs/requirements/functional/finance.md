# Financeiro — `FIN`

Do orçamento ao recebimento, o caixa, as contas da clínica e a nota fiscal.

## FR-FIN-01 — Orçamento com aceite

> Como recepção, quero registrar que o paciente aceitou o orçamento, para que a cobrança nasça do que foi aceito.

O sistema deve registrar o aceite do paciente num orçamento.

| Atributo | Valor |
| --- | --- |
| Justificativa | O aceite é o segundo passo do funil que a gestão mede. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-FIN-01.1** — Dado um orçamento apresentado, quando o aceite é registrado, então o orçamento passa a aceito com a data do aceite.

## FR-FIN-02 — Cobrança parcelada

> Como recepção, quero gerar a cobrança de um orçamento aceito em parcelas, para que o paciente pague como combinado.

O sistema deve gerar, a partir de um orçamento aceito, a cobrança em uma ou mais parcelas com vencimento.

| Atributo | Valor |
| --- | --- |
| Justificativa | O pagamento parcelado faz parte do fluxo de cobrança da clínica. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-FIN-02.1** — Dado um orçamento aceito em 3 parcelas, quando a cobrança é gerada, então existem 3 parcelas com vencimento e valor.

## FR-FIN-03 — Recebimento com forma de pagamento

> Como recepção, quero registrar o pagamento de uma parcela e a forma usada, para que o saldo do paciente esteja correto.

O sistema deve registrar o recebimento de uma parcela com a forma de pagamento.

| Atributo | Valor |
| --- | --- |
| Justificativa | O recebimento fecha a parcela e alimenta o caixa. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-FIN-03.1** — Dada uma parcela em aberto, quando o recebimento é registrado, então a parcela passa a paga e mostra a forma de pagamento.

## FR-FIN-04 — Caixa

> Como recepção, quero abrir e fechar o caixa do dia, para que o dinheiro do dia seja conferido.

O sistema deve manter o caixa da clínica, com as entradas e saídas do período e o saldo.

| Atributo | Valor |
| --- | --- |
| Justificativa | O caixa mostra o dinheiro que entrou e saiu no período. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-FIN-04.1** — Dado um recebimento em dinheiro, quando o caixa do dia é aberto, então o recebimento aparece como entrada.

## FR-FIN-05 — Contas a pagar

> Como gestor, quero registrar as despesas da clínica e seus vencimentos, para que eu saiba o que vence.

O sistema deve manter as contas a pagar da clínica, com valor, vencimento e situação.

| Atributo | Valor |
| --- | --- |
| Justificativa | As despesas da clínica são controladas no mesmo sistema. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-FIN-05.1** — Dada uma conta a pagar com vencimento, quando o gestor lista as contas do mês, então ela aparece com a situação.

## FR-FIN-06 — Contas a receber

> Como gestor, quero ver o que a clínica tem a receber, para que eu preveja a entrada de dinheiro.

O sistema deve apresentar as contas a receber da clínica a partir das parcelas em aberto.

| Atributo | Valor |
| --- | --- |
| Justificativa | As parcelas em aberto mostram o que a clínica tem a receber. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-FIN-06.1** — Dadas parcelas em aberto, quando o gestor abre contas a receber, então vê cada parcela com paciente, valor e vencimento.

## FR-FIN-07 — Repasse e comissão de dentista

> Como gestor, quero calcular o que cabe a cada dentista, para que o repasse seja pago sem planilha.

O sistema deve calcular o repasse de cada dentista a partir dos procedimentos realizados por ele e da regra de comissão configurada.

| Atributo | Valor |
| --- | --- |
| Justificativa | O repasse do dentista é calculado no sistema, não fora dele. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-FIN-07.1** — Dado um dentista com regra de comissão e procedimentos recebidos no período, quando o gestor abre o repasse, então vê o valor devido a ele.

## FR-FIN-08 — Inadimplência

> Como gestor, quero ver as parcelas vencidas e não pagas, para que eu cobre o paciente.

O sistema deve apresentar as parcelas vencidas sem recebimento, por paciente.

| Atributo | Valor |
| --- | --- |
| Justificativa | A parcela vencida precisa aparecer para ser cobrada. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-FIN-08.1** — Dado um orçamento aceito e parcelado, quando uma parcela vence sem pagamento, então ela aparece na inadimplência.

## FR-FIN-09 — Emissão de NFS-e

> Como gestor, quero emitir a nota fiscal de serviço de um recebimento, para que a clínica cumpra a obrigação fiscal sem outro sistema.

O sistema deve emitir a NFS-e de um recebimento e mantê-la vinculada a ele.

| Atributo | Valor |
| --- | --- |
| Justificativa | A nota emitida fora do sistema quebra o fluxo de ponta a ponta. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-FIN-09.1** — Dado um recebimento, quando o gestor emite a nota, então a NFS-e é gerada e fica vinculada ao recebimento.

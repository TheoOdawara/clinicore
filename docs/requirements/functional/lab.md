# Laboratório protético — `LAB`

Os pedidos enviados ao laboratório protético, controlados manualmente.

## FR-LAB-01 — Cadastro de laboratórios

> Como responsável por laboratório, quero cadastrar os laboratórios com que a clínica trabalha, para que o pedido aponte para um deles.

O sistema deve manter o cadastro de laboratórios protéticos.

| Atributo | Valor |
| --- | --- |
| Justificativa | O pedido aponta para um laboratório cadastrado. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-LAB-01.1** — Dado um laboratório cadastrado, quando um pedido é criado, então o laboratório pode ser escolhido.

## FR-LAB-02 — Pedido ao laboratório

> Como responsável por laboratório, quero registrar o pedido com paciente, tratamento, prazo e situação, para que eu saiba o que está fora e quando volta.

O sistema deve registrar o pedido ao laboratório vinculado ao paciente e ao tratamento, com envio, prazo, retorno e situação, lançados manualmente.

| Atributo | Valor |
| --- | --- |
| Justificativa | A clínica precisa ver o atraso de um pedido antes da consulta do paciente. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-LAB-02.1** — Dado um pedido enviado, quando o prazo passa sem retorno registrado, então o pedido aparece como atrasado.

## FR-LAB-03 — Custo do pedido no financeiro

> Como gestor, quero que o custo do pedido entre nas contas a pagar, para que o resultado do tratamento considere o laboratório.

O sistema deve lançar o custo de cada pedido ao laboratório como conta a pagar da clínica.

| Atributo | Valor |
| --- | --- |
| Justificativa | O custo do laboratório entra no financeiro da clínica. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-LAB-03.1** — Dado um pedido com custo informado, quando é registrado, então existe uma conta a pagar com o mesmo valor vinculada ao pedido.

# Convênios e TISS — `INS`

O atendimento por convênio odontológico, faturado por arquivo no padrão TISS.

## FR-INS-01 — Operadoras e tabelas

> Como gestor, quero cadastrar as operadoras e suas tabelas de procedimentos, para que o atendimento por convênio use o valor contratado.

O sistema deve manter o cadastro de operadoras de convênio e a tabela de procedimentos de cada uma.

| Atributo | Valor |
| --- | --- |
| Justificativa | O valor do procedimento por convênio vem da tabela da operadora. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-INS-01.1** — Dada uma operadora com tabela, quando um atendimento por ela é registrado, então o valor vem da tabela dela.

## FR-INS-02 — Guias e lotes em arquivo TISS

> Como gestor, quero gerar as guias e os lotes no padrão TISS, para que eu fature o convênio sem redigitar.

O sistema deve gerar guias e lotes em arquivo no padrão TISS para as operadoras do primeiro corte.

| Atributo | Valor |
| --- | --- |
| Justificativa | Sem integração direta, o arquivo TISS é a via de faturamento. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-INS-02.1** — Dado um atendimento por convênio, quando é faturado, então o sistema gera o arquivo TISS aceito pela operadora.

## FR-INS-03 — Glosa e recurso

> Como gestor, quero registrar a glosa de uma guia e o recurso apresentado, para que eu acompanhe o que a operadora não pagou.

O sistema deve permitir registrar manualmente a glosa de uma guia e o recurso, com a situação de cada um.

| Atributo | Valor |
| --- | --- |
| Justificativa | O recurso de uma glosa precisa de acompanhamento até o pagamento. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-INS-03.1** — Dada uma guia glosada, quando a glosa é registrada, então a guia mostra o valor glosado e a situação do recurso.

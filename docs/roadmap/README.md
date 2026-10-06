# Roadmap

O plano: cada marco dividido em épicos, e qual épico entrega cada requisito. O estado de cada item não
fica aqui: está no [GitHub Project](https://github.com/users/TheoOdawara/projects/3) e nos
[milestones](https://github.com/TheoOdawara/clinicore/milestones).

## M1 — Go-live piloto

A primeira clínica piloto sai do Simples Dental e opera no Clinicore. GitHub Milestone:
[Go-live piloto](https://github.com/TheoOdawara/clinicore/milestone/1).

| Épico | Issue | Cobre |
| --- | --- | --- |
| Plataforma | #1 | [`NFR-PERF-01` e `02`](../requirements/non-functional/performance.md), [`NFR-INTR-01` a `03`](../requirements/non-functional/interaction.md), [`NFR-REL-01`](../requirements/non-functional/reliability.md), [`NFR-SEC-02`](../requirements/non-functional/security.md), [`NFR-FLEX-01`](../requirements/non-functional/flexibility.md) |
| Autenticação e sessão | #3 | [`FR-ACC-07` a `12`](../requirements/functional/access.md), [`NFR-FLEX-02`](../requirements/non-functional/flexibility.md) |
| Organização e acesso | #5 | [`FR-ACC-01` a `06`](../requirements/functional/access.md), [`NFR-SEC-01` e `03`](../requirements/non-functional/security.md) |
| Pacientes | #10 | [`FR-PAT-01` a `03`](../requirements/functional/patients.md) |
| Agenda | #13 | [`FR-SCH-01` a `08`](../requirements/functional/scheduling.md) |
| Prontuário clínico | #17 | [`FR-CLN-01` a `10`](../requirements/functional/clinical-record.md) |
| Financeiro | #26 | [`FR-FIN-01` a `09`](../requirements/functional/finance.md) |
| Estoque | #32 | [`FR-STK-01` a `05`](../requirements/functional/inventory.md) |
| Prontuário estético | #36 | [`FR-AES-01` a `05`](../requirements/functional/aesthetic-record.md) |
| Laboratório protético | #40 | [`FR-LAB-01` a `03`](../requirements/functional/lab.md) |
| Indicadores de gestão | #43 | [`FR-KPI-01` a `06`](../requirements/functional/indicators.md) |
| Offline no app | #144 | [`NFR-REL-02` e `03`](../requirements/non-functional/reliability.md), [`NFR-SEC-04`](../requirements/non-functional/security.md) |

Os 75 requisitos aprovados do M1 estão na tabela, cada um em um épico só. As regras de negócio (`BR-*`)
não têm marco: valem dentro do épico que toca o que elas regem.

A #114, que migra a API para Rust, e a #91, que leva a autenticação ao app, entregam os requisitos da #3
em outra implementação e em outra superfície; não cobrem requisito próprio.

## Marcos seguintes

O M2 e o M3 são divididos em épicos quando o M1 fechar. O que já existe no backlog: o épico #90 para o
M2; #46, #50 e #53 para o M3, que ainda não tem épico para a landing (`FR-SITE-*`).

## Sprints

- [Sprint 1](sprints/sprint-01.md) — criar a conta, entrar, recuperar a senha e sair pelo navegador

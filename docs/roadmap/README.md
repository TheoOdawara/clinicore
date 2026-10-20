# Roadmap

O plano: cada marco dividido em épicos, e qual épico entrega cada requisito. O estado de cada item não
fica aqui: está no [GitHub Project](https://github.com/users/TheoOdawara/projects/3) e nos
[milestones](https://github.com/TheoOdawara/clinicore/milestones).

## M1 — Go-live piloto

A primeira clínica piloto sai do Simples Dental e opera no Clinicore. GitHub Milestone:
[Go-live piloto](https://github.com/TheoOdawara/clinicore/milestone/1).

| Épico | Issue | Cobre |
| --- | --- | --- |
| Plataforma | #1 | [`NFR-PERF-01` e `02`](../requirements/non-functional/performance.md), [`NFR-INTR-01` a `03`](../requirements/non-functional/interaction.md), [`NFR-REL-01` e `04`](../requirements/non-functional/reliability.md), [`NFR-SEC-02` e `05`](../requirements/non-functional/security.md), [`NFR-FLEX-01`](../requirements/non-functional/flexibility.md) |
| Autenticação e sessão | #3 | [`FR-ACC-07` a `12` e `15`](../requirements/functional/access.md), [`NFR-FLEX-02`](../requirements/non-functional/flexibility.md), [`NFR-SEC-07`](../requirements/non-functional/security.md) |
| Organização e acesso | #5 | [`FR-ACC-01` a `06` e `16` a `19`](../requirements/functional/access.md), [`NFR-SEC-01`](../requirements/non-functional/security.md) |
| Pacientes | #10 | [`FR-PAT-01` a `03`](../requirements/functional/patients.md) |
| Agenda | #13 | [`FR-SCH-01` a `10`](../requirements/functional/scheduling.md) |
| Prontuário clínico | #17 | [`FR-CLN-01` a `11`](../requirements/functional/clinical-record.md) |
| Financeiro | #26 | [`FR-FIN-01` a `10`](../requirements/functional/finance.md) |
| Estoque | #32 | [`FR-STK-01` a `06`](../requirements/functional/inventory.md) |
| Prontuário estético | #36 | [`FR-AES-01` a `07`](../requirements/functional/aesthetic-record.md) |
| Laboratório protético | #40 | [`FR-LAB-01` a `03`](../requirements/functional/lab.md) |
| Indicadores de gestão | #43 | [`FR-KPI-01` a `07`](../requirements/functional/indicators.md) |
| Offline no app | #144 | [`NFR-REL-02`, `03` e `05`](../requirements/non-functional/reliability.md), [`NFR-SEC-04` e `06`](../requirements/non-functional/security.md) |

Os 92 requisitos aprovados do M1 estão na tabela, cada um em um épico só. As regras de negócio (`BR-*`)
não têm marco: valem dentro do épico que toca o que elas regem.

A #114, que migra a API para Rust, e a #91, que leva a autenticação ao app, entregam os requisitos da #3
em outra implementação e em outra superfície; não cobrem requisito próprio.

## M3 — Pós go-live

Convênios, portabilidade, IA, landing e passkey, depois que a piloto opera no Clinicore. GitHub
Milestone: [Pós go-live](https://github.com/TheoOdawara/clinicore/milestone/2).

O M3 é dividido em épicos quando o M1 fechar. O que já existe no backlog: #46 para convênios, #50 para
portabilidade, #53 para IA e #90 para passkey (`FR-ACC-13`, `14`, `20` e `21`). A landing (`FR-SITE-*`) ainda não
tem épico.

O M2, Beta, foi fundido no M3 na v2.0.0 do SRS.

## Cobertura de requisitos

Todo requisito aprovado de um marco planejado está em exatamente um épico. Hoje só o M1 está planejado.

| Requisito | Épico | Marco |
| --- | --- | --- |
| [`NFR-PERF-01`](../requirements/non-functional/performance.md#nfr-perf-01--tempo-de-resposta-da-api) | #1 | M1 |
| [`NFR-PERF-02`](../requirements/non-functional/performance.md#nfr-perf-02--tempo-de-exibição-da-tela) | #1 | M1 |
| [`NFR-INTR-01`](../requirements/non-functional/interaction.md#nfr-intr-01--mobile-first) | #1 | M1 |
| [`NFR-INTR-02`](../requirements/non-functional/interaction.md#nfr-intr-02--acessibilidade) | #1 | M1 |
| [`NFR-INTR-03`](../requirements/non-functional/interaction.md#nfr-intr-03--interface-em-pt-br) | #1 | M1 |
| [`NFR-REL-01`](../requirements/non-functional/reliability.md#nfr-rel-01--disponibilidade) | #1 | M1 |
| [`NFR-REL-04`](../requirements/non-functional/reliability.md#nfr-rel-04--publicação-sem-indisponibilidade) | #1 | M1 |
| [`NFR-SEC-02`](../requirements/non-functional/security.md#nfr-sec-02--criptografia-em-trânsito) | #1 | M1 |
| [`NFR-SEC-05`](../requirements/non-functional/security.md#nfr-sec-05--criptografia-em-repouso) | #1 | M1 |
| [`NFR-FLEX-01`](../requirements/non-functional/flexibility.md#nfr-flex-01--web-e-app-nativo-com-as-mesmas-telas) | #1 | M1 |
| [`FR-ACC-07`](../requirements/functional/access.md#fr-acc-07--criar-conta-com-e-mail-e-senha) | #3 | M1 |
| [`FR-ACC-08`](../requirements/functional/access.md#fr-acc-08--entrar-com-e-mail-e-senha) | #3 | M1 |
| [`FR-ACC-09`](../requirements/functional/access.md#fr-acc-09--entrar-com-google) | #3 | M1 |
| [`FR-ACC-10`](../requirements/functional/access.md#fr-acc-10--recuperar-a-senha) | #3 | M1 |
| [`FR-ACC-11`](../requirements/functional/access.md#fr-acc-11--trocar-a-senha) | #3 | M1 |
| [`FR-ACC-12`](../requirements/functional/access.md#fr-acc-12--sair-e-encerrar-a-sessão) | #3 | M1 |
| [`FR-ACC-15`](../requirements/functional/access.md#fr-acc-15--confirmar-o-e-mail-antes-do-primeiro-acesso) | #3 | M1 |
| [`NFR-FLEX-02`](../requirements/non-functional/flexibility.md#nfr-flex-02--web-instalável) | #3 | M1 |
| [`NFR-SEC-07`](../requirements/non-functional/security.md#nfr-sec-07--nenhuma-resposta-da-api-no-cache-do-navegador) | #3 | M1 |
| [`FR-ACC-01`](../requirements/functional/access.md#fr-acc-01--cliente-como-clínica-rede-ou-dentista-autônomo) | #5 | M1 |
| [`FR-ACC-02`](../requirements/functional/access.md#fr-acc-02--cliente-pessoa-física-ou-jurídica) | #5 | M1 |
| [`FR-ACC-03`](../requirements/functional/access.md#fr-acc-03--papéis-com-permissões-distintas) | #5 | M1 |
| [`FR-ACC-04`](../requirements/functional/access.md#fr-acc-04--gestor-limitado-à-própria-clínica) | #5 | M1 |
| [`FR-ACC-05`](../requirements/functional/access.md#fr-acc-05--desativar-usuário) | #5 | M1 |
| [`FR-ACC-06`](../requirements/functional/access.md#fr-acc-06--consulta-da-trilha-de-acesso-ao-prontuário) | #5 | M1 |
| [`FR-ACC-16`](../requirements/functional/access.md#fr-acc-16--dono-de-rede-alcança-todas-as-clínicas) | #5 | M1 |
| [`FR-ACC-17`](../requirements/functional/access.md#fr-acc-17--convidar-usuário) | #5 | M1 |
| [`FR-ACC-18`](../requirements/functional/access.md#fr-acc-18--alterar-o-papel-do-usuário) | #5 | M1 |
| [`FR-ACC-19`](../requirements/functional/access.md#fr-acc-19--registro-de-acesso-ao-prontuário) | #5 | M1 |
| [`NFR-SEC-01`](../requirements/non-functional/security.md#nfr-sec-01--isolamento-entre-clientes) | #5 | M1 |
| [`FR-PAT-01`](../requirements/functional/patients.md#fr-pat-01--cadastro-de-paciente) | #10 | M1 |
| [`FR-PAT-02`](../requirements/functional/patients.md#fr-pat-02--responsável-legal-para-menor) | #10 | M1 |
| [`FR-PAT-03`](../requirements/functional/patients.md#fr-pat-03--histórico-único-do-paciente) | #10 | M1 |
| [`FR-SCH-01`](../requirements/functional/scheduling.md#fr-sch-01--agenda-por-dentista) | #13 | M1 |
| [`FR-SCH-02`](../requirements/functional/scheduling.md#fr-sch-02--agenda-por-cadeira-ou-sala) | #13 | M1 |
| [`FR-SCH-03`](../requirements/functional/scheduling.md#fr-sch-03--pedido-de-confirmação-por-whatsapp) | #13 | M1 |
| [`FR-SCH-04`](../requirements/functional/scheduling.md#fr-sch-04--lembrete-por-whatsapp) | #13 | M1 |
| [`FR-SCH-05`](../requirements/functional/scheduling.md#fr-sch-05--registro-de-falta) | #13 | M1 |
| [`FR-SCH-06`](../requirements/functional/scheduling.md#fr-sch-06--remarcação) | #13 | M1 |
| [`FR-SCH-07`](../requirements/functional/scheduling.md#fr-sch-07--encaixe) | #13 | M1 |
| [`FR-SCH-08`](../requirements/functional/scheduling.md#fr-sch-08--inclusão-na-lista-de-espera) | #13 | M1 |
| [`FR-SCH-09`](../requirements/functional/scheduling.md#fr-sch-09--resposta-da-confirmação-na-agenda) | #13 | M1 |
| [`FR-SCH-10`](../requirements/functional/scheduling.md#fr-sch-10--lista-de-espera-no-horário-liberado) | #13 | M1 |
| [`FR-CLN-01`](../requirements/functional/clinical-record.md#fr-cln-01--anamnese) | #17 | M1 |
| [`FR-CLN-02`](../requirements/functional/clinical-record.md#fr-cln-02--odontograma) | #17 | M1 |
| [`FR-CLN-03`](../requirements/functional/clinical-record.md#fr-cln-03--plano-de-tratamento) | #17 | M1 |
| [`FR-CLN-04`](../requirements/functional/clinical-record.md#fr-cln-04--evolução-clínica-por-atendimento) | #17 | M1 |
| [`FR-CLN-05`](../requirements/functional/clinical-record.md#fr-cln-05--emissão-de-documentos) | #17 | M1 |
| [`FR-CLN-06`](../requirements/functional/clinical-record.md#fr-cln-06--impressão-de-documentos) | #17 | M1 |
| [`FR-CLN-07`](../requirements/functional/clinical-record.md#fr-cln-07--assinatura-qualificada-do-dentista) | #17 | M1 |
| [`FR-CLN-08`](../requirements/functional/clinical-record.md#fr-cln-08--assinatura-avançada-do-dentista) | #17 | M1 |
| [`FR-CLN-09`](../requirements/functional/clinical-record.md#fr-cln-09--assinatura-do-paciente) | #17 | M1 |
| [`FR-CLN-10`](../requirements/functional/clinical-record.md#fr-cln-10--imagens-e-exames-anexados) | #17 | M1 |
| [`FR-CLN-11`](../requirements/functional/clinical-record.md#fr-cln-11--orçamento-gerado-do-plano) | #17 | M1 |
| [`FR-FIN-01`](../requirements/functional/finance.md#fr-fin-01--orçamento-com-aceite) | #26 | M1 |
| [`FR-FIN-02`](../requirements/functional/finance.md#fr-fin-02--cobrança-parcelada) | #26 | M1 |
| [`FR-FIN-03`](../requirements/functional/finance.md#fr-fin-03--recebimento-com-forma-de-pagamento) | #26 | M1 |
| [`FR-FIN-04`](../requirements/functional/finance.md#fr-fin-04--caixa) | #26 | M1 |
| [`FR-FIN-10`](../requirements/functional/finance.md#fr-fin-10--saldo-do-caixa) | #26 | M1 |
| [`FR-FIN-05`](../requirements/functional/finance.md#fr-fin-05--contas-a-pagar) | #26 | M1 |
| [`FR-FIN-06`](../requirements/functional/finance.md#fr-fin-06--contas-a-receber) | #26 | M1 |
| [`FR-FIN-07`](../requirements/functional/finance.md#fr-fin-07--repasse-e-comissão-de-dentista) | #26 | M1 |
| [`FR-FIN-08`](../requirements/functional/finance.md#fr-fin-08--inadimplência) | #26 | M1 |
| [`FR-FIN-09`](../requirements/functional/finance.md#fr-fin-09--emissão-de-nfs-e) | #26 | M1 |
| [`FR-STK-01`](../requirements/functional/inventory.md#fr-stk-01--cadastro-de-materiais-e-produtos) | #32 | M1 |
| [`FR-STK-02`](../requirements/functional/inventory.md#fr-stk-02--entradas-e-saídas) | #32 | M1 |
| [`FR-STK-03`](../requirements/functional/inventory.md#fr-stk-03--baixa-do-produto-no-procedimento) | #32 | M1 |
| [`FR-STK-04`](../requirements/functional/inventory.md#fr-stk-04--alerta-de-estoque-baixo) | #32 | M1 |
| [`FR-STK-05`](../requirements/functional/inventory.md#fr-stk-05--alerta-de-validade-próxima) | #32 | M1 |
| [`FR-STK-06`](../requirements/functional/inventory.md#fr-stk-06--saldo-por-lote) | #32 | M1 |
| [`FR-AES-01`](../requirements/functional/aesthetic-record.md#fr-aes-01--registro-de-procedimento-de-harmonização) | #36 | M1 |
| [`FR-AES-02`](../requirements/functional/aesthetic-record.md#fr-aes-02--mapa-facial) | #36 | M1 |
| [`FR-AES-03`](../requirements/functional/aesthetic-record.md#fr-aes-03--rastreabilidade-do-produto-aplicado) | #36 | M1 |
| [`FR-AES-04`](../requirements/functional/aesthetic-record.md#fr-aes-04--fotos-de-antes-e-de-depois) | #36 | M1 |
| [`FR-AES-05`](../requirements/functional/aesthetic-record.md#fr-aes-05--termo-de-consentimento-de-estética) | #36 | M1 |
| [`FR-AES-06`](../requirements/functional/aesthetic-record.md#fr-aes-06--comparação-de-antes-e-depois) | #36 | M1 |
| [`FR-AES-07`](../requirements/functional/aesthetic-record.md#fr-aes-07--busca-por-lote-aplicado) | #36 | M1 |
| [`FR-LAB-01`](../requirements/functional/lab.md#fr-lab-01--cadastro-de-laboratórios) | #40 | M1 |
| [`FR-LAB-02`](../requirements/functional/lab.md#fr-lab-02--pedido-ao-laboratório) | #40 | M1 |
| [`FR-LAB-03`](../requirements/functional/lab.md#fr-lab-03--custo-do-pedido-no-financeiro) | #40 | M1 |
| [`FR-KPI-01`](../requirements/functional/indicators.md#fr-kpi-01--funil-de-orçamento) | #43 | M1 |
| [`FR-KPI-02`](../requirements/functional/indicators.md#fr-kpi-02--ocupação-de-cadeira-e-de-dentista) | #43 | M1 |
| [`FR-KPI-03`](../requirements/functional/indicators.md#fr-kpi-03--taxa-de-faltas) | #43 | M1 |
| [`FR-KPI-04`](../requirements/functional/indicators.md#fr-kpi-04--faturamento-e-inadimplência) | #43 | M1 |
| [`FR-KPI-05`](../requirements/functional/indicators.md#fr-kpi-05--pacientes-inativos-e-reativados) | #43 | M1 |
| [`FR-KPI-06`](../requirements/functional/indicators.md#fr-kpi-06--indicadores-por-clínica) | #43 | M1 |
| [`FR-KPI-07`](../requirements/functional/indicators.md#fr-kpi-07--indicadores-consolidados-da-rede) | #43 | M1 |
| [`NFR-REL-02`](../requirements/non-functional/reliability.md#nfr-rel-02--consulta-offline-no-app) | #144 | M1 |
| [`NFR-REL-03`](../requirements/non-functional/reliability.md#nfr-rel-03--conflito-offline-nunca-sobrescreve) | #144 | M1 |
| [`NFR-REL-05`](../requirements/non-functional/reliability.md#nfr-rel-05--gravação-offline-no-app) | #144 | M1 |
| [`NFR-SEC-04`](../requirements/non-functional/security.md#nfr-sec-04--dado-de-saúde-no-aparelho) | #144 | M1 |
| [`NFR-SEC-06`](../requirements/non-functional/security.md#nfr-sec-06--dado-de-saúde-apagado-do-aparelho) | #144 | M1 |

## Sprints

- [Sprint 1](sprints/sprint-01.md) — criar a conta, entrar, recuperar a senha e sair pelo navegador
- [Sprint 2](sprints/sprint-02.md) — a mesma meta, levada da Sprint 1

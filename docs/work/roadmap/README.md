# Roadmap

O plano: cada marco dividido em épicos, e qual épico entrega cada requisito. O estado de cada item não
fica aqui: está no [GitHub Project](https://github.com/users/TheoOdawara/projects/3) e nos
[milestones](https://github.com/TheoOdawara/clinicore/milestones).

O Project tem só o que está planejado na sprint atual. O resto do roadmap mora neste arquivo e nos
milestones. Nenhum épico tem issue ainda (—): cada um ganha a sua quando entra numa sprint, com uma
issue por app.

## M1 — Go-live piloto

A primeira clínica piloto sai do Simples Dental e opera no Clinicore. O GitHub Milestone nasce
no planejamento da primeira sprint.

| Épico | Issue | Cobre |
| --- | --- | --- |
| Plataforma | — | [`NFR-PERF-01` e `02`](../../product/requirements/non-functional/performance.md), [`NFR-INTR-01` a `03`](../../product/requirements/non-functional/interaction.md), [`NFR-REL-01` e `04`](../../product/requirements/non-functional/reliability.md), [`NFR-SEC-02` e `05`](../../product/requirements/non-functional/security.md), [`NFR-FLEX-01`](../../product/requirements/non-functional/flexibility.md) |
| Autenticação e sessão | — | [`FR-ACC-07` a `12` e `15`](../../product/requirements/functional/access.md), [`NFR-FLEX-02`](../../product/requirements/non-functional/flexibility.md), [`NFR-SEC-07`](../../product/requirements/non-functional/security.md) |
| Organização e acesso | — | [`FR-ACC-01` a `06` e `16` a `19`](../../product/requirements/functional/access.md), [`NFR-SEC-01`](../../product/requirements/non-functional/security.md) |
| Pacientes | — | [`FR-PAT-01` a `03`](../../product/requirements/functional/patients.md) |
| Agenda | — | [`FR-SCH-01` a `10`](../../product/requirements/functional/scheduling.md) |
| Prontuário clínico | — | [`FR-CLN-01` a `11`](../../product/requirements/functional/clinical-record.md) |
| Financeiro | — | [`FR-FIN-01` a `10`](../../product/requirements/functional/finance.md) |
| Estoque | — | [`FR-STK-01` a `06`](../../product/requirements/functional/inventory.md) |
| Prontuário estético | — | [`FR-AES-01` a `07`](../../product/requirements/functional/aesthetic-record.md) |
| Laboratório protético | — | [`FR-LAB-01` a `03`](../../product/requirements/functional/lab.md) |
| Indicadores de gestão | — | [`FR-KPI-01` a `07`](../../product/requirements/functional/indicators.md) |
| Offline no app | — | [`NFR-REL-02`, `03` e `05`](../../product/requirements/non-functional/reliability.md), [`NFR-SEC-04` e `06`](../../product/requirements/non-functional/security.md) |

Os 92 requisitos aprovados do M1 estão na tabela, cada um em um épico só. As regras de negócio (`BR-*`)
não têm marco: valem dentro do épico que toca o que elas regem.

## M3 — Pós go-live

Convênios, portabilidade, IA, landing e passkey, depois que a piloto opera no Clinicore.

O M3 é dividido em épicos quando o M1 fechar.

O M2, Beta, foi fundido no M3 na v2.0.0 do SRS.

## Cobertura de requisitos

Todo requisito aprovado de um marco planejado está em exatamente um épico. Hoje só o M1 está planejado.

| Requisito | Épico | Marco |
| --- | --- | --- |
| [`NFR-PERF-01`](../../product/requirements/non-functional/performance.md#nfr-perf-01--tempo-de-resposta-da-api) | Plataforma | M1 |
| [`NFR-PERF-02`](../../product/requirements/non-functional/performance.md#nfr-perf-02--tempo-de-exibição-da-tela) | Plataforma | M1 |
| [`NFR-INTR-01`](../../product/requirements/non-functional/interaction.md#nfr-intr-01--mobile-first) | Plataforma | M1 |
| [`NFR-INTR-02`](../../product/requirements/non-functional/interaction.md#nfr-intr-02--acessibilidade) | Plataforma | M1 |
| [`NFR-INTR-03`](../../product/requirements/non-functional/interaction.md#nfr-intr-03--interface-em-pt-br) | Plataforma | M1 |
| [`NFR-REL-01`](../../product/requirements/non-functional/reliability.md#nfr-rel-01--disponibilidade) | Plataforma | M1 |
| [`NFR-REL-04`](../../product/requirements/non-functional/reliability.md#nfr-rel-04--publicação-sem-indisponibilidade) | Plataforma | M1 |
| [`NFR-SEC-02`](../../product/requirements/non-functional/security.md#nfr-sec-02--criptografia-em-trânsito) | Plataforma | M1 |
| [`NFR-SEC-05`](../../product/requirements/non-functional/security.md#nfr-sec-05--criptografia-em-repouso) | Plataforma | M1 |
| [`NFR-FLEX-01`](../../product/requirements/non-functional/flexibility.md#nfr-flex-01--web-e-app-nativo-com-as-mesmas-telas) | Plataforma | M1 |
| [`FR-ACC-07`](../../product/requirements/functional/access.md#fr-acc-07--criar-conta-com-e-mail-e-senha) | Autenticação e sessão | M1 |
| [`FR-ACC-08`](../../product/requirements/functional/access.md#fr-acc-08--entrar-com-e-mail-e-senha) | Autenticação e sessão | M1 |
| [`FR-ACC-09`](../../product/requirements/functional/access.md#fr-acc-09--entrar-com-google) | Autenticação e sessão | M1 |
| [`FR-ACC-10`](../../product/requirements/functional/access.md#fr-acc-10--recuperar-a-senha) | Autenticação e sessão | M1 |
| [`FR-ACC-11`](../../product/requirements/functional/access.md#fr-acc-11--trocar-a-senha) | Autenticação e sessão | M1 |
| [`FR-ACC-12`](../../product/requirements/functional/access.md#fr-acc-12--sair-e-encerrar-a-sessão) | Autenticação e sessão | M1 |
| [`FR-ACC-15`](../../product/requirements/functional/access.md#fr-acc-15--confirmar-o-e-mail-antes-do-primeiro-acesso) | Autenticação e sessão | M1 |
| [`NFR-FLEX-02`](../../product/requirements/non-functional/flexibility.md#nfr-flex-02--web-instalável) | Autenticação e sessão | M1 |
| [`NFR-SEC-07`](../../product/requirements/non-functional/security.md#nfr-sec-07--nenhuma-resposta-da-api-no-cache-do-navegador) | Autenticação e sessão | M1 |
| [`FR-ACC-01`](../../product/requirements/functional/access.md#fr-acc-01--cliente-como-clínica-rede-ou-dentista-autônomo) | Organização e acesso | M1 |
| [`FR-ACC-02`](../../product/requirements/functional/access.md#fr-acc-02--cliente-pessoa-física-ou-jurídica) | Organização e acesso | M1 |
| [`FR-ACC-03`](../../product/requirements/functional/access.md#fr-acc-03--papéis-com-permissões-distintas) | Organização e acesso | M1 |
| [`FR-ACC-04`](../../product/requirements/functional/access.md#fr-acc-04--gestor-limitado-à-própria-clínica) | Organização e acesso | M1 |
| [`FR-ACC-05`](../../product/requirements/functional/access.md#fr-acc-05--desativar-usuário) | Organização e acesso | M1 |
| [`FR-ACC-06`](../../product/requirements/functional/access.md#fr-acc-06--consulta-da-trilha-de-acesso-ao-prontuário) | Organização e acesso | M1 |
| [`FR-ACC-16`](../../product/requirements/functional/access.md#fr-acc-16--dono-de-rede-alcança-todas-as-clínicas) | Organização e acesso | M1 |
| [`FR-ACC-17`](../../product/requirements/functional/access.md#fr-acc-17--convidar-usuário) | Organização e acesso | M1 |
| [`FR-ACC-18`](../../product/requirements/functional/access.md#fr-acc-18--alterar-o-papel-do-usuário) | Organização e acesso | M1 |
| [`FR-ACC-19`](../../product/requirements/functional/access.md#fr-acc-19--registro-de-acesso-ao-prontuário) | Organização e acesso | M1 |
| [`NFR-SEC-01`](../../product/requirements/non-functional/security.md#nfr-sec-01--isolamento-entre-clientes) | Organização e acesso | M1 |
| [`FR-PAT-01`](../../product/requirements/functional/patients.md#fr-pat-01--cadastro-de-paciente) | Pacientes | M1 |
| [`FR-PAT-02`](../../product/requirements/functional/patients.md#fr-pat-02--responsável-legal-para-menor) | Pacientes | M1 |
| [`FR-PAT-03`](../../product/requirements/functional/patients.md#fr-pat-03--histórico-único-do-paciente) | Pacientes | M1 |
| [`FR-SCH-01`](../../product/requirements/functional/scheduling.md#fr-sch-01--agenda-por-dentista) | Agenda | M1 |
| [`FR-SCH-02`](../../product/requirements/functional/scheduling.md#fr-sch-02--agenda-por-cadeira-ou-sala) | Agenda | M1 |
| [`FR-SCH-03`](../../product/requirements/functional/scheduling.md#fr-sch-03--pedido-de-confirmação-por-whatsapp) | Agenda | M1 |
| [`FR-SCH-04`](../../product/requirements/functional/scheduling.md#fr-sch-04--lembrete-por-whatsapp) | Agenda | M1 |
| [`FR-SCH-05`](../../product/requirements/functional/scheduling.md#fr-sch-05--registro-de-falta) | Agenda | M1 |
| [`FR-SCH-06`](../../product/requirements/functional/scheduling.md#fr-sch-06--remarcação) | Agenda | M1 |
| [`FR-SCH-07`](../../product/requirements/functional/scheduling.md#fr-sch-07--encaixe) | Agenda | M1 |
| [`FR-SCH-08`](../../product/requirements/functional/scheduling.md#fr-sch-08--inclusão-na-lista-de-espera) | Agenda | M1 |
| [`FR-SCH-09`](../../product/requirements/functional/scheduling.md#fr-sch-09--resposta-da-confirmação-na-agenda) | Agenda | M1 |
| [`FR-SCH-10`](../../product/requirements/functional/scheduling.md#fr-sch-10--lista-de-espera-no-horário-liberado) | Agenda | M1 |
| [`FR-CLN-01`](../../product/requirements/functional/clinical-record.md#fr-cln-01--anamnese) | Prontuário clínico | M1 |
| [`FR-CLN-02`](../../product/requirements/functional/clinical-record.md#fr-cln-02--odontograma) | Prontuário clínico | M1 |
| [`FR-CLN-03`](../../product/requirements/functional/clinical-record.md#fr-cln-03--plano-de-tratamento) | Prontuário clínico | M1 |
| [`FR-CLN-04`](../../product/requirements/functional/clinical-record.md#fr-cln-04--evolução-clínica-por-atendimento) | Prontuário clínico | M1 |
| [`FR-CLN-05`](../../product/requirements/functional/clinical-record.md#fr-cln-05--emissão-de-documentos) | Prontuário clínico | M1 |
| [`FR-CLN-06`](../../product/requirements/functional/clinical-record.md#fr-cln-06--impressão-de-documentos) | Prontuário clínico | M1 |
| [`FR-CLN-07`](../../product/requirements/functional/clinical-record.md#fr-cln-07--assinatura-qualificada-do-dentista) | Prontuário clínico | M1 |
| [`FR-CLN-08`](../../product/requirements/functional/clinical-record.md#fr-cln-08--assinatura-avançada-do-dentista) | Prontuário clínico | M1 |
| [`FR-CLN-09`](../../product/requirements/functional/clinical-record.md#fr-cln-09--assinatura-do-paciente) | Prontuário clínico | M1 |
| [`FR-CLN-10`](../../product/requirements/functional/clinical-record.md#fr-cln-10--imagens-e-exames-anexados) | Prontuário clínico | M1 |
| [`FR-CLN-11`](../../product/requirements/functional/clinical-record.md#fr-cln-11--orçamento-gerado-do-plano) | Prontuário clínico | M1 |
| [`FR-FIN-01`](../../product/requirements/functional/finance.md#fr-fin-01--orçamento-com-aceite) | Financeiro | M1 |
| [`FR-FIN-02`](../../product/requirements/functional/finance.md#fr-fin-02--cobrança-parcelada) | Financeiro | M1 |
| [`FR-FIN-03`](../../product/requirements/functional/finance.md#fr-fin-03--recebimento-com-forma-de-pagamento) | Financeiro | M1 |
| [`FR-FIN-04`](../../product/requirements/functional/finance.md#fr-fin-04--caixa) | Financeiro | M1 |
| [`FR-FIN-10`](../../product/requirements/functional/finance.md#fr-fin-10--saldo-do-caixa) | Financeiro | M1 |
| [`FR-FIN-05`](../../product/requirements/functional/finance.md#fr-fin-05--contas-a-pagar) | Financeiro | M1 |
| [`FR-FIN-06`](../../product/requirements/functional/finance.md#fr-fin-06--contas-a-receber) | Financeiro | M1 |
| [`FR-FIN-07`](../../product/requirements/functional/finance.md#fr-fin-07--repasse-e-comissão-de-dentista) | Financeiro | M1 |
| [`FR-FIN-08`](../../product/requirements/functional/finance.md#fr-fin-08--inadimplência) | Financeiro | M1 |
| [`FR-FIN-09`](../../product/requirements/functional/finance.md#fr-fin-09--emissão-de-nfs-e) | Financeiro | M1 |
| [`FR-STK-01`](../../product/requirements/functional/inventory.md#fr-stk-01--cadastro-de-materiais-e-produtos) | Estoque | M1 |
| [`FR-STK-02`](../../product/requirements/functional/inventory.md#fr-stk-02--entradas-e-saídas) | Estoque | M1 |
| [`FR-STK-03`](../../product/requirements/functional/inventory.md#fr-stk-03--baixa-do-produto-no-procedimento) | Estoque | M1 |
| [`FR-STK-04`](../../product/requirements/functional/inventory.md#fr-stk-04--alerta-de-estoque-baixo) | Estoque | M1 |
| [`FR-STK-05`](../../product/requirements/functional/inventory.md#fr-stk-05--alerta-de-validade-próxima) | Estoque | M1 |
| [`FR-STK-06`](../../product/requirements/functional/inventory.md#fr-stk-06--saldo-por-lote) | Estoque | M1 |
| [`FR-AES-01`](../../product/requirements/functional/aesthetic-record.md#fr-aes-01--registro-de-procedimento-de-harmonização) | Prontuário estético | M1 |
| [`FR-AES-02`](../../product/requirements/functional/aesthetic-record.md#fr-aes-02--mapa-facial) | Prontuário estético | M1 |
| [`FR-AES-03`](../../product/requirements/functional/aesthetic-record.md#fr-aes-03--rastreabilidade-do-produto-aplicado) | Prontuário estético | M1 |
| [`FR-AES-04`](../../product/requirements/functional/aesthetic-record.md#fr-aes-04--fotos-de-antes-e-de-depois) | Prontuário estético | M1 |
| [`FR-AES-05`](../../product/requirements/functional/aesthetic-record.md#fr-aes-05--termo-de-consentimento-de-estética) | Prontuário estético | M1 |
| [`FR-AES-06`](../../product/requirements/functional/aesthetic-record.md#fr-aes-06--comparação-de-antes-e-depois) | Prontuário estético | M1 |
| [`FR-AES-07`](../../product/requirements/functional/aesthetic-record.md#fr-aes-07--busca-por-lote-aplicado) | Prontuário estético | M1 |
| [`FR-LAB-01`](../../product/requirements/functional/lab.md#fr-lab-01--cadastro-de-laboratórios) | Laboratório protético | M1 |
| [`FR-LAB-02`](../../product/requirements/functional/lab.md#fr-lab-02--pedido-ao-laboratório) | Laboratório protético | M1 |
| [`FR-LAB-03`](../../product/requirements/functional/lab.md#fr-lab-03--custo-do-pedido-no-financeiro) | Laboratório protético | M1 |
| [`FR-KPI-01`](../../product/requirements/functional/indicators.md#fr-kpi-01--funil-de-orçamento) | Indicadores de gestão | M1 |
| [`FR-KPI-02`](../../product/requirements/functional/indicators.md#fr-kpi-02--ocupação-de-cadeira-e-de-dentista) | Indicadores de gestão | M1 |
| [`FR-KPI-03`](../../product/requirements/functional/indicators.md#fr-kpi-03--taxa-de-faltas) | Indicadores de gestão | M1 |
| [`FR-KPI-04`](../../product/requirements/functional/indicators.md#fr-kpi-04--faturamento-e-inadimplência) | Indicadores de gestão | M1 |
| [`FR-KPI-05`](../../product/requirements/functional/indicators.md#fr-kpi-05--pacientes-inativos-e-reativados) | Indicadores de gestão | M1 |
| [`FR-KPI-06`](../../product/requirements/functional/indicators.md#fr-kpi-06--indicadores-por-clínica) | Indicadores de gestão | M1 |
| [`FR-KPI-07`](../../product/requirements/functional/indicators.md#fr-kpi-07--indicadores-consolidados-da-rede) | Indicadores de gestão | M1 |
| [`NFR-REL-02`](../../product/requirements/non-functional/reliability.md#nfr-rel-02--consulta-offline-no-app) | Offline no app | M1 |
| [`NFR-REL-03`](../../product/requirements/non-functional/reliability.md#nfr-rel-03--conflito-offline-nunca-sobrescreve) | Offline no app | M1 |
| [`NFR-REL-05`](../../product/requirements/non-functional/reliability.md#nfr-rel-05--gravação-offline-no-app) | Offline no app | M1 |
| [`NFR-SEC-04`](../../product/requirements/non-functional/security.md#nfr-sec-04--dado-de-saúde-no-aparelho) | Offline no app | M1 |
| [`NFR-SEC-06`](../../product/requirements/non-functional/security.md#nfr-sec-06--dado-de-saúde-apagado-do-aparelho) | Offline no app | M1 |

## Sprints

Nenhuma ainda. A primeira é planejada pelo `/tightship:sprint plan`.

# Clinicore — Especificação de Requisitos de Software

**Versão:** 1.0.0 · **Data:** 2026-10-03 · [Changelog](CHANGELOG.md)

> Clinicore é codinome interno. O nome comercial está em aberto ([OQ-01](open-questions.md#oq-01--nome-comercial)).

## Propósito

Clínicas odontológicas operam com sistemas legados (Clinicorp, Simples Dental, softwares desktop
antigos) ou com papel e planilha. Nenhum cobre a clínica de ponta a ponta:

- **Não é de ponta a ponta** — agenda, clínico, financeiro, convênio e atendimento ficam espalhados
  entre ferramentas.
- **Estética ignorada** — a harmonização orofacial é forte na odontologia e quase não tem suporte.
- **UX ruim** — telas lentas, confusas, pensadas para desktop.
- **Aprisionamento de dados** — sair do sistema é inviável: a exportação é incompleta ou vem errada.

O Clinicore existe para eliminar o papel da clínica até onde a regulação permite, cobrir o fluxo
inteiro num só lugar e devolver ao cliente o controle dos próprios dados.

## Escopo

**Dentro do MVP:** organização e acesso, pacientes, agenda, prontuário clínico, prontuário estético,
financeiro com NFS-e, estoque, laboratório protético, indicadores de gestão, convênios por arquivo
TISS, portabilidade, IA no atendimento, IA clínica e a landing pública.

**Fora do escopo:**

| Item | Por quê |
| --- | --- |
| App do paciente e autoagendamento | App de clínica única não tem adoção; só faria sentido como marketplace ([BR-10](business-rules.md#br-10--o-paciente-não-opera-o-sistema)) |
| Marketplace de agendamento entre clínicas | Não é este produto |
| Especialidades não odontológicas | O produto é só odontologia |
| Integração direta com operadoras de convênio | O faturamento é por arquivo TISS |
| Integração com laboratórios protéticos | O controle é manual |
| Cobrança do SaaS: planos, preço e assinatura | Vem depois do sistema ([OQ-08](open-questions.md#oq-08--modelo-comercial-do-saas)) |

## Marcos

| Marco | Nome | O que marca | No GitHub |
| --- | --- | --- | --- |
| M1 | Go-live piloto | O corte mínimo para a primeira clínica piloto sair do Simples Dental | milestone `Go-live piloto` |
| M2 | Beta | O que entra quando a piloto usa o sistema no dia a dia | épico #90 |
| M3 | Pós go-live | Convênios, portabilidade, IA e landing | milestone `Pós go-live` |

A importação da clínica piloto é feita à mão no onboarding conduzido pelo Theo; por isso a
portabilidade fica no M3.

**Prioridade:** o que o M1 exige é `Must`; o que entra no M2 e no M3 é `Should`.

## Conteúdo

[Visão geral](overview.md) · [Glossário](glossary.md) · [Regras de negócio](business-rules.md) ·
[Perguntas em aberto](open-questions.md)

- [Organização e acesso](functional/access.md) — `FR-ACC`
- [Pacientes](functional/patients.md) — `FR-PAT`
- [Agenda](functional/scheduling.md) — `FR-SCH`
- [Prontuário clínico](functional/clinical-record.md) — `FR-CLN`
- [Prontuário estético](functional/aesthetic-record.md) — `FR-AES`
- [Financeiro](functional/finance.md) — `FR-FIN`
- [Convênios e TISS](functional/insurance.md) — `FR-INS`
- [Portabilidade](functional/portability.md) — `FR-PRT`
- [IA no atendimento](functional/ai-reception.md) — `FR-AIA`
- [IA clínica](functional/ai-clinical.md) — `FR-AIC`
- [Indicadores de gestão](functional/indicators.md) — `FR-KPI`
- [Estoque](functional/inventory.md) — `FR-STK`
- [Laboratório protético](functional/lab.md) — `FR-LAB`
- [Landing pública](functional/site.md) — `FR-SITE`
- [Eficiência de desempenho](non-functional/performance.md) — `NFR-PERF`
- [Capacidade de interação](non-functional/interaction.md) — `NFR-INTR`
- [Confiabilidade](non-functional/reliability.md) — `NFR-REL`
- [Segurança](non-functional/security.md) — `NFR-SEC`
- [Flexibilidade](non-functional/flexibility.md) — `NFR-FLEX`

## Requisitos

| ID | Nome | Prioridade | Status | Marco |
| --- | --- | --- | --- | --- |
| [FR-ACC-01](functional/access.md#fr-acc-01--cliente-como-clínica-rede-ou-dentista-autônomo) | Cliente como clínica, rede ou dentista autônomo | Must | approved | M1 |
| [FR-ACC-02](functional/access.md#fr-acc-02--cliente-pessoa-física-ou-jurídica) | Cliente pessoa física ou jurídica | Must | approved | M1 |
| [FR-ACC-03](functional/access.md#fr-acc-03--papéis-com-permissões-distintas) | Papéis com permissões distintas | Must | approved | M1 |
| [FR-ACC-04](functional/access.md#fr-acc-04--alcance-por-clínica-dentro-da-rede) | Alcance por clínica dentro da rede | Must | approved | M1 |
| [FR-ACC-05](functional/access.md#fr-acc-05--gestão-de-usuários-da-clínica) | Gestão de usuários da clínica | Must | approved | M1 |
| [FR-ACC-06](functional/access.md#fr-acc-06--trilha-de-auditoria-de-acesso-ao-prontuário) | Trilha de auditoria de acesso ao prontuário | Must | approved | M1 |
| [FR-ACC-07](functional/access.md#fr-acc-07--criar-conta-com-e-mail-e-senha) | Criar conta com e-mail e senha | Must | approved | M1 |
| [FR-ACC-08](functional/access.md#fr-acc-08--entrar-com-e-mail-e-senha) | Entrar com e-mail e senha | Must | approved | M1 |
| [FR-ACC-09](functional/access.md#fr-acc-09--entrar-com-google) | Entrar com Google | Must | approved | M1 |
| [FR-ACC-10](functional/access.md#fr-acc-10--recuperar-a-senha) | Recuperar a senha | Must | approved | M1 |
| [FR-ACC-11](functional/access.md#fr-acc-11--trocar-a-senha) | Trocar a senha | Must | approved | M1 |
| [FR-ACC-12](functional/access.md#fr-acc-12--sair-e-encerrar-a-sessão) | Sair e encerrar a sessão | Must | approved | M1 |
| [FR-ACC-13](functional/access.md#fr-acc-13--entrar-com-passkey) | Entrar com passkey | Should | approved | M2 |
| [FR-ACC-14](functional/access.md#fr-acc-14--gerenciar-passkeys) | Gerenciar passkeys | Should | approved | M2 |
| [FR-PAT-01](functional/patients.md#fr-pat-01--cadastro-de-paciente) | Cadastro de paciente | Must | approved | M1 |
| [FR-PAT-02](functional/patients.md#fr-pat-02--responsável-legal-para-menor) | Responsável legal para menor | Must | approved | M1 |
| [FR-PAT-03](functional/patients.md#fr-pat-03--histórico-único-do-paciente) | Histórico único do paciente | Must | approved | M1 |
| [FR-SCH-01](functional/scheduling.md#fr-sch-01--agenda-por-dentista) | Agenda por dentista | Must | approved | M1 |
| [FR-SCH-02](functional/scheduling.md#fr-sch-02--agenda-por-cadeira-ou-sala) | Agenda por cadeira ou sala | Must | approved | M1 |
| [FR-SCH-03](functional/scheduling.md#fr-sch-03--confirmação-por-whatsapp) | Confirmação por WhatsApp | Must | approved | M1 |
| [FR-SCH-04](functional/scheduling.md#fr-sch-04--lembrete-por-whatsapp) | Lembrete por WhatsApp | Must | approved | M1 |
| [FR-SCH-05](functional/scheduling.md#fr-sch-05--registro-de-falta) | Registro de falta | Must | approved | M1 |
| [FR-SCH-06](functional/scheduling.md#fr-sch-06--remarcação) | Remarcação | Must | approved | M1 |
| [FR-SCH-07](functional/scheduling.md#fr-sch-07--encaixe) | Encaixe | Must | approved | M1 |
| [FR-SCH-08](functional/scheduling.md#fr-sch-08--lista-de-espera) | Lista de espera | Must | approved | M1 |
| [FR-CLN-01](functional/clinical-record.md#fr-cln-01--anamnese) | Anamnese | Must | approved | M1 |
| [FR-CLN-02](functional/clinical-record.md#fr-cln-02--odontograma) | Odontograma | Must | approved | M1 |
| [FR-CLN-03](functional/clinical-record.md#fr-cln-03--plano-de-tratamento-com-orçamento) | Plano de tratamento com orçamento | Must | approved | M1 |
| [FR-CLN-04](functional/clinical-record.md#fr-cln-04--evolução-clínica-por-atendimento) | Evolução clínica por atendimento | Must | approved | M1 |
| [FR-CLN-05](functional/clinical-record.md#fr-cln-05--emissão-de-documentos) | Emissão de documentos | Must | approved | M1 |
| [FR-CLN-06](functional/clinical-record.md#fr-cln-06--impressão-de-documentos) | Impressão de documentos | Must | approved | M1 |
| [FR-CLN-07](functional/clinical-record.md#fr-cln-07--assinatura-qualificada-do-dentista) | Assinatura qualificada do dentista | Must | approved | M1 |
| [FR-CLN-08](functional/clinical-record.md#fr-cln-08--assinatura-avançada-do-dentista) | Assinatura avançada do dentista | Must | approved | M1 |
| [FR-CLN-09](functional/clinical-record.md#fr-cln-09--assinatura-do-paciente) | Assinatura do paciente | Must | approved | M1 |
| [FR-CLN-10](functional/clinical-record.md#fr-cln-10--imagens-e-exames-anexados) | Imagens e exames anexados | Must | approved | M1 |
| [FR-AES-01](functional/aesthetic-record.md#fr-aes-01--registro-de-procedimento-de-harmonização) | Registro de procedimento de harmonização | Must | approved | M1 |
| [FR-AES-02](functional/aesthetic-record.md#fr-aes-02--mapa-facial) | Mapa facial | Must | approved | M1 |
| [FR-AES-03](functional/aesthetic-record.md#fr-aes-03--rastreabilidade-do-produto-aplicado) | Rastreabilidade do produto aplicado | Must | approved | M1 |
| [FR-AES-04](functional/aesthetic-record.md#fr-aes-04--fotos-de-antes-e-depois) | Fotos de antes e depois | Must | approved | M1 |
| [FR-AES-05](functional/aesthetic-record.md#fr-aes-05--termo-de-consentimento-de-estética) | Termo de consentimento de estética | Must | approved | M1 |
| [FR-FIN-01](functional/finance.md#fr-fin-01--orçamento-com-aceite) | Orçamento com aceite | Must | approved | M1 |
| [FR-FIN-02](functional/finance.md#fr-fin-02--cobrança-parcelada) | Cobrança parcelada | Must | approved | M1 |
| [FR-FIN-03](functional/finance.md#fr-fin-03--recebimento-com-forma-de-pagamento) | Recebimento com forma de pagamento | Must | approved | M1 |
| [FR-FIN-04](functional/finance.md#fr-fin-04--caixa) | Caixa | Must | approved | M1 |
| [FR-FIN-05](functional/finance.md#fr-fin-05--contas-a-pagar) | Contas a pagar | Must | approved | M1 |
| [FR-FIN-06](functional/finance.md#fr-fin-06--contas-a-receber) | Contas a receber | Must | approved | M1 |
| [FR-FIN-07](functional/finance.md#fr-fin-07--repasse-e-comissão-de-dentista) | Repasse e comissão de dentista | Must | approved | M1 |
| [FR-FIN-08](functional/finance.md#fr-fin-08--inadimplência) | Inadimplência | Must | approved | M1 |
| [FR-FIN-09](functional/finance.md#fr-fin-09--emissão-de-nfs-e) | Emissão de NFS-e | Must | approved | M1 |
| [FR-INS-01](functional/insurance.md#fr-ins-01--operadoras-e-tabelas) | Operadoras e tabelas | Should | approved | M3 |
| [FR-INS-02](functional/insurance.md#fr-ins-02--guias-e-lotes-em-arquivo-tiss) | Guias e lotes em arquivo TISS | Should | approved | M3 |
| [FR-INS-03](functional/insurance.md#fr-ins-03--glosa-e-recurso) | Glosa e recurso | Should | approved | M3 |
| [FR-PRT-01](functional/portability.md#fr-prt-01--importação-do-simples-dental) | Importação do Simples Dental | Should | approved | M3 |
| [FR-PRT-02](functional/portability.md#fr-prt-02--relatório-de-importação) | Relatório de importação | Should | approved | M3 |
| [FR-PRT-03](functional/portability.md#fr-prt-03--exportação-completa-self-service) | Exportação completa self-service | Should | approved | M3 |
| [FR-AIA-01](functional/ai-reception.md#fr-aia-01--agente-de-agendamento-no-whatsapp) | Agente de agendamento no WhatsApp | Should | approved | M3 |
| [FR-AIA-02](functional/ai-reception.md#fr-aia-02--reativação-de-pacientes-e-de-orçamentos) | Reativação de pacientes e de orçamentos | Should | approved | M3 |
| [FR-AIA-03](functional/ai-reception.md#fr-aia-03--passagem-para-humano) | Passagem para humano | Should | approved | M3 |
| [FR-AIC-01](functional/ai-clinical.md#fr-aic-01--rascunho-de-evolução-por-transcrição) | Rascunho de evolução por transcrição | Should | approved | M3 |
| [FR-AIC-02](functional/ai-clinical.md#fr-aic-02--revisão-e-aprovação-do-rascunho) | Revisão e aprovação do rascunho | Should | approved | M3 |
| [FR-KPI-01](functional/indicators.md#fr-kpi-01--funil-de-orçamento) | Funil de orçamento | Must | approved | M1 |
| [FR-KPI-02](functional/indicators.md#fr-kpi-02--ocupação-de-cadeira-e-de-dentista) | Ocupação de cadeira e de dentista | Must | approved | M1 |
| [FR-KPI-03](functional/indicators.md#fr-kpi-03--taxa-de-faltas) | Taxa de faltas | Must | approved | M1 |
| [FR-KPI-04](functional/indicators.md#fr-kpi-04--faturamento-e-inadimplência) | Faturamento e inadimplência | Must | approved | M1 |
| [FR-KPI-05](functional/indicators.md#fr-kpi-05--pacientes-inativos-e-reativados) | Pacientes inativos e reativados | Must | approved | M1 |
| [FR-KPI-06](functional/indicators.md#fr-kpi-06--visão-por-clínica-e-consolidada-por-rede) | Visão por clínica e consolidada por rede | Must | approved | M1 |
| [FR-STK-01](functional/inventory.md#fr-stk-01--cadastro-de-materiais-e-produtos) | Cadastro de materiais e produtos | Must | approved | M1 |
| [FR-STK-02](functional/inventory.md#fr-stk-02--entradas-saídas-e-saldo) | Entradas, saídas e saldo | Must | approved | M1 |
| [FR-STK-03](functional/inventory.md#fr-stk-03--baixa-do-produto-no-procedimento) | Baixa do produto no procedimento | Must | approved | M1 |
| [FR-STK-04](functional/inventory.md#fr-stk-04--alerta-de-estoque-baixo) | Alerta de estoque baixo | Must | approved | M1 |
| [FR-STK-05](functional/inventory.md#fr-stk-05--alerta-de-validade-próxima) | Alerta de validade próxima | Must | approved | M1 |
| [FR-LAB-01](functional/lab.md#fr-lab-01--cadastro-de-laboratórios) | Cadastro de laboratórios | Must | approved | M1 |
| [FR-LAB-02](functional/lab.md#fr-lab-02--pedido-ao-laboratório) | Pedido ao laboratório | Must | approved | M1 |
| [FR-LAB-03](functional/lab.md#fr-lab-03--custo-do-pedido-no-financeiro) | Custo do pedido no financeiro | Must | approved | M1 |
| [FR-SITE-01](functional/site.md#fr-site-01--apresentação-do-produto) | Apresentação do produto | Should | approved | M3 |
| [FR-SITE-02](functional/site.md#fr-site-02--entrada-no-sistema-pela-landing) | Entrada no sistema pela landing | Should | approved | M3 |
| [FR-SITE-03](functional/site.md#fr-site-03--captação-de-interessados) | Captação de interessados | Should | approved | M3 |
| [FR-SITE-04](functional/site.md#fr-site-04--preços-e-planos) | Preços e planos | Should | approved | M3 |
| [NFR-PERF-01](non-functional/performance.md#nfr-perf-01--tempo-de-resposta-da-api) | Tempo de resposta da API | Must | approved | M1 |
| [NFR-PERF-02](non-functional/performance.md#nfr-perf-02--tempo-de-exibição-da-tela) | Tempo de exibição da tela | Must | approved | M1 |
| [NFR-INTR-01](non-functional/interaction.md#nfr-intr-01--mobile-first) | Mobile-first | Must | approved | M1 |
| [NFR-INTR-02](non-functional/interaction.md#nfr-intr-02--acessibilidade) | Acessibilidade | Must | approved | M1 |
| [NFR-INTR-03](non-functional/interaction.md#nfr-intr-03--interface-em-pt-br) | Interface em pt-BR | Must | approved | M1 |
| [NFR-REL-01](non-functional/reliability.md#nfr-rel-01--disponibilidade) | Disponibilidade | Must | approved | M1 |
| [NFR-REL-02](non-functional/reliability.md#nfr-rel-02--operação-offline-no-app) | Operação offline no app | Must | approved | M1 |
| [NFR-REL-03](non-functional/reliability.md#nfr-rel-03--conflito-offline-nunca-sobrescreve) | Conflito offline nunca sobrescreve | Must | approved | M1 |
| [NFR-SEC-01](non-functional/security.md#nfr-sec-01--isolamento-entre-clientes) | Isolamento entre clientes | Must | approved | M1 |
| [NFR-SEC-02](non-functional/security.md#nfr-sec-02--criptografia-em-trânsito-e-em-repouso) | Criptografia em trânsito e em repouso | Must | approved | M1 |
| [NFR-SEC-03](non-functional/security.md#nfr-sec-03--menor-privilégio-por-papel) | Menor privilégio por papel | Must | approved | M1 |
| [NFR-SEC-04](non-functional/security.md#nfr-sec-04--dado-de-saúde-no-aparelho) | Dado de saúde no aparelho | Must | approved | M1 |
| [NFR-FLEX-01](non-functional/flexibility.md#nfr-flex-01--web-e-app-nativo-com-as-mesmas-telas) | Web e app nativo com as mesmas telas | Must | approved | M1 |
| [NFR-FLEX-02](non-functional/flexibility.md#nfr-flex-02--web-instalável) | Web instalável | Should | approved | M1 |
| [BR-01](business-rules.md#br-01--assinatura-qualificada-para-atestado-e-receita-de-controle-especial) | Assinatura qualificada para atestado e receita de controle especial | Must | approved | — |
| [BR-02](business-rules.md#br-02--assinatura-avançada-nos-demais-documentos-do-profissional) | Assinatura avançada nos demais documentos do profissional | Must | approved | — |
| [BR-03](business-rules.md#br-03--prevalência-da-assinatura-qualificada) | Prevalência da assinatura qualificada | Must | approved | — |
| [BR-04](business-rules.md#br-04--prontuário-assinado-pelo-dentista-e-pelo-paciente) | Prontuário assinado pelo dentista e pelo paciente | Must | approved | — |
| [BR-05](business-rules.md#br-05--papel-dispensado-só-com-icp-brasil) | Papel dispensado só com ICP-Brasil | Must | approved | — |
| [BR-06](business-rules.md#br-06--impressão-sempre-disponível) | Impressão sempre disponível | Must | approved | — |
| [BR-07](business-rules.md#br-07--nada-de-ia-entra-no-prontuário-sem-aprovação) | Nada de IA entra no prontuário sem aprovação | Must | approved | — |
| [BR-08](business-rules.md#br-08--gravação-só-com-consentimento) | Gravação só com consentimento | Must | approved | — |
| [BR-09](business-rules.md#br-09--exportação-nunca-bloqueada) | Exportação nunca bloqueada | Must | approved | — |
| [BR-10](business-rules.md#br-10--o-paciente-não-opera-o-sistema) | O paciente não opera o sistema | Must | approved | — |

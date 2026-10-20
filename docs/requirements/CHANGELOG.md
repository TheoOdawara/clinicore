# Changelog

Uma entrada por versão do SRS: a data, os IDs adicionados, alterados ou aposentados, e o porquê.

## 2.0.0 — 2026-10-19

Revisão do SRS contra o workflow de documentação atual e das decisões do produto (#151).

- **Marco movido:** o M2, Beta, foi fundido no M3. `FR-ACC-13` e `FR-ACC-14`, passkey, passam ao M3.
- **Aposentado:** `NFR-SEC-03`, que duplicava o `FR-ACC-03`. O critério dele virou o `FR-ACC-03.2`.
- **Divididos**, por terem duas capacidades num requisito. O original mantém o ID e uma das
  capacidades, e cada outra recebe o próximo ID livre do grupo:
  - `FR-ACC-04` → `FR-ACC-16`, o dono de rede alcança todas as clínicas;
  - `FR-ACC-05` → `FR-ACC-17` e `18`, convidar e alterar o papel; o `05` fica com desativar;
  - `FR-ACC-06` → `FR-ACC-19`, o registro do acesso; o `06` fica com a consulta;
  - `FR-ACC-07` → `FR-ACC-15`, confirmação do e-mail antes do primeiro acesso;
  - `FR-ACC-14` → `FR-ACC-20` e `21`, registrar e listar passkeys; o `14` fica com remover;
  - `FR-SCH-03` → `FR-SCH-09`, a resposta da confirmação na agenda;
  - `FR-SCH-08` → `FR-SCH-10`, a lista de espera no horário liberado;
  - `FR-CLN-03` → `FR-CLN-11`, o orçamento gerado do plano;
  - `FR-AES-04` → `FR-AES-06`, a comparação de antes e depois;
  - `FR-STK-02` → `FR-STK-06`, o saldo por lote;
  - `FR-AIA-01` → `FR-AIA-04` e `05`, confirmar e agendar; o `01` fica com remarcar;
  - `FR-AIA-02` → `FR-AIA-06`, a retomada de orçamentos não aprovados;
  - `FR-KPI-06` → `FR-KPI-07`, os indicadores consolidados da rede;
  - `FR-FIN-04` → `FR-FIN-10`, o saldo do caixa; o `04` fica com abrir e fechar o caixa do dia;
  - `NFR-REL-02` → `NFR-REL-05`, a gravação offline; o `02` fica com a consulta;
  - `NFR-SEC-02` → `NFR-SEC-05`, criptografia em repouso, no servidor;
  - `NFR-SEC-04` → `NFR-SEC-06`, dado de saúde apagado do aparelho;
  - `NFR-FLEX-02` → `NFR-SEC-07`, nenhuma resposta da API no cache do navegador.
- **Adicionados:**
  - `NFR-REL-04`, publicação da API e do web sem indisponibilidade: a clínica opera o dia inteiro;
  - `FR-AES-07`, busca por lote aplicado, que era o critério do `FR-AES-03` sem estar no texto dele.
- **Alterados:**
  - `NFR-FLEX-02` passa de `Should` a `Must`, já que tudo no M1 é `Must`; a origem deixa de citar a
    issue #81 e passa a nomear o Theo, e a justificativa perde a parte do cache, que foi para o
    `NFR-SEC-07`.
  - "Manter" virou um verbo testável em `FR-PAT-01`, `FR-SCH-01`, `FR-SCH-02`, `FR-CLN-01`,
    `FR-CLN-02`, `FR-FIN-04`, `FR-FIN-05`, `FR-INS-01`, `FR-STK-01` e `FR-LAB-01`.
  - Critérios refeitos para testar o texto do próprio requisito: `FR-ACC-03.2`, `FR-AES-03.1`,
    `FR-KPI-01.1`, `NFR-REL-02.1` e `NFR-FLEX-02.1`.
  - O `FR-ACC-08` deixa a confirmação do e-mail só para o `FR-ACC-15`.
  - O `NFR-SEC-06` só apaga a fila pendente na sessão revogada ou com a confirmação do usuário: no
    logout, o app envia a fila antes de sair. A ADR 0007 foi emendada no item 7.
  - O `FR-FIN-07` calcula o repasse pelo procedimento realizado, sem o desconto dado pela clínica, que
    o novo critério `FR-FIN-07.2` testa.
  - O `FR-SITE-04` volta a `proposed`: os preços dependem do modelo comercial da `OQ-08`, ainda em
    aberto.
  - As janelas de confirmação e de lembrete (`FR-SCH-03`, `FR-SCH-04`) passam a ser a antecedência
    configurada pela clínica.
  - Texto do requisito sem detalhe de implementação: `FR-ACC-12.1`, `FR-SITE-02`, `NFR-REL-03.1` e
    `NFR-SEC-05.1`.
  - O papel de estoque e laboratório tem um nome só nas histórias de `FR-STK-*` e `FR-LAB-*`.
  - A origem de `FR-ACC-07` a `14` passa a nomear o Theo, com a spec como registro.
- **Glossário:** entram caixa, guia, recurso, lote de produto, lote TISS, paciente inativo, paciente
  reativado e sessão de assinatura; saem PWA, tenant e p95, que nenhum requisito usa; LCP passa a
  Largest Contentful Paint, o nome que o `NFR-PERF-02` usa.
- **Perguntas em aberto:** a `OQ-01` ganha o candidato Core Clinical System.
- **Forma:** toda lista ganhou uma linha em branco antes, para o site local de documentação
  renderizá-la como lista. O índice deixa a prioridade das regras de negócio em branco, como na
  tabela de cada uma.
- **Diagramas:** cada área funcional ganhou o diagrama UML de casos de uso, em
  [`diagrams/`](diagrams/README.md).

## 1.0.0 — 2026-10-03

Primeira versão do SRS em pasta, a partir do `docs/requirements.md` de arquivo único (#139).

- **Adicionados:** `FR-ACC-01` a `14`, `FR-PAT-01` a `03`, `FR-SCH-01` a `08`, `FR-CLN-01` a `10`,
  `FR-AES-01` a `05`, `FR-FIN-01` a `09`, `FR-INS-01` a `03`, `FR-PRT-01` a `03`, `FR-AIA-01` a `03`,
  `FR-AIC-01` e `02`, `FR-KPI-01` a `06`, `FR-STK-01` a `05`, `FR-LAB-01` a `03`, `FR-SITE-01` a `04`,
  `NFR-PERF-01` e `02`, `NFR-INTR-01` a `03`, `NFR-REL-01` a `03`, `NFR-SEC-01` a `04`, `NFR-FLEX-01`
  e `02`, `BR-01` a `10`.
- **Novos em relação ao arquivo único:** autenticação (`FR-ACC-07` a `14`), que estava especificada e
  sem requisito; landing pública (`FR-SITE-*`); web instalável (`NFR-FLEX-02`); metas de desempenho
  (`NFR-PERF-*`), de disponibilidade (`NFR-REL-01`) e de acessibilidade (`NFR-INTR-02`).
- **Reclassificados:** a trilha de auditoria do prontuário passou de não funcional a `FR-ACC-06`; as
  exigências legais de assinatura viraram `BR-01` a `BR-06`; a baixa do produto estético no estoque,
  que aparecia em três lugares, é só `FR-STK-03`.
- **Saiu do SRS:** a escolha de provedores de assinatura digital, hoje na
  [ADR 0013](../decisions/0013-assinatura-digital-em-duas-camadas.md).
- **Perguntas em aberto novas:** `OQ-09` a `OQ-11`.

## Antes da 1.0.0

O histórico do arquivo único, sem versão:

- **2026-09-15** — Kickoff: problema, usuários, escopo do MVP, multi-tenant com redes, app do paciente
  fora de escopo. No mesmo dia: corte do go-live da clínica piloto; NFS-e, estoque e laboratório
  protético entram no MVP e no go-live; portabilidade sai do go-live; assinatura digital pesquisada e
  dividida em duas camadas; WhatsApp pela Twilio; TISS só por arquivo; cobrança do SaaS adiada.
- **2026-09-21** — App nativo de iOS e de Android entra no MVP e no go-live, com offline completo
  ([ADR 0007](../decisions/0007-app-nativo-em-flutter-com-offline.md)).

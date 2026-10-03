# Changelog

Uma entrada por versão do SRS: a data, os IDs adicionados, alterados ou aposentados, e o porquê.

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

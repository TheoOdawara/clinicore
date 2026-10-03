# IA clínica — `AIC`

A transcrição da consulta que vira rascunho de evolução.

## FR-AIC-01 — Rascunho de evolução por transcrição

> Como dentista, quero receber um rascunho da evolução a partir da gravação da consulta, para que eu não digite o que acabei de falar.

O sistema deve transcrever a consulta gravada e gerar um rascunho de evolução clínica.

| Atributo | Valor |
| --- | --- |
| Justificativa | A transcrição poupa o dentista de digitar a evolução. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-AIC-01.1** — Dada uma consulta gravada com consentimento, quando ela termina, então o dentista recebe um rascunho de evolução.

## FR-AIC-02 — Revisão e aprovação do rascunho

> Como dentista, quero revisar, editar e aprovar o rascunho, para que só entre no prontuário o que eu validei.

O sistema deve permitir que o dentista edite, aprove ou descarte o rascunho antes de ele entrar no prontuário.

| Atributo | Valor |
| --- | --- |
| Justificativa | O prontuário é responsabilidade do dentista (BR-07). |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Should |
| Status | approved |
| Marco | M3 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-AIC-02.1** — Dado um rascunho não aprovado, quando o prontuário é aberto, então o rascunho não faz parte dele.
- **FR-AIC-02.2** — Dado um rascunho aprovado pelo dentista, quando o prontuário é aberto, então a evolução aparece com o dentista como autor.

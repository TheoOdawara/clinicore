# Regras de negócio — `BR`

Políticas que valem independentemente da funcionalidade. Um requisito que depende de uma delas a cita pelo ID.

## BR-01 — Assinatura qualificada para atestado e receita de controle especial

Atestado e receita de controle especial só são assinados eletronicamente com assinatura qualificada ICP-Brasil.

| Atributo | Valor |
| --- | --- |
| Justificativa | Sem a qualificada, o documento não tem validade. |
| Origem | Lei 14.063/2020, art. 13 |
| Status | approved |
| Desde | v1.0.0 |

## BR-02 — Assinatura avançada nos demais documentos do profissional

Os demais documentos do profissional de saúde são assinados eletronicamente com assinatura avançada ou qualificada.

| Atributo | Valor |
| --- | --- |
| Justificativa | Permite eliminar parte do papel para o dentista sem certificado. |
| Origem | Lei 14.063/2020, art. 14 |
| Status | approved |
| Desde | v1.0.0 |

## BR-03 — Prevalência da assinatura qualificada

Quando duas normas divergem sobre o tipo de assinatura de um documento, vale a assinatura qualificada.

| Atributo | Valor |
| --- | --- |
| Justificativa | Resolve o conflito entre a Lei 14.063 e as resoluções do CFO. |
| Origem | Lei 14.063/2020, art. 5º, §5º |
| Status | approved |
| Desde | v1.0.0 |

## BR-04 — Prontuário assinado pelo dentista e pelo paciente

O prontuário leva a assinatura do dentista e a do paciente ou do responsável legal.

| Atributo | Valor |
| --- | --- |
| Justificativa | Exigência do conselho profissional. |
| Origem | Resolução CFO 278/2025, art. 8º, §1º |
| Status | approved |
| Desde | v1.0.0 |

## BR-05 — Papel dispensado só com ICP-Brasil

O papel do prontuário só é dispensado quando o prontuário eletrônico atende ao nível NGS2, que exige certificado ICP-Brasil.

| Atributo | Valor |
| --- | --- |
| Justificativa | Define até onde o sistema pode eliminar o papel hoje. |
| Origem | Resolução CFO 91/2009 |
| Status | approved |
| Desde | v1.0.0 |

## BR-06 — Impressão sempre disponível

Todo documento do paciente pode ser impresso; o que não pode ser assinado eletronicamente é impresso e assinado à mão.

| Atributo | Valor |
| --- | --- |
| Justificativa | Nenhum atendimento para por falta de assinatura eletrônica. |
| Origem | Theo, kickoff de 2026-09-15 |
| Status | approved |
| Desde | v1.0.0 |

## BR-07 — Nada de IA entra no prontuário sem aprovação

Nenhum conteúdo gerado por IA entra no prontuário sem a aprovação do dentista responsável.

| Atributo | Valor |
| --- | --- |
| Justificativa | O prontuário é responsabilidade profissional do dentista. |
| Origem | Theo, kickoff de 2026-09-15 |
| Status | approved |
| Desde | v1.0.0 |

## BR-08 — Gravação só com consentimento

A consulta só é gravada ou transcrita com o consentimento do paciente registrado antes da gravação.

| Atributo | Valor |
| --- | --- |
| Justificativa | Áudio de consulta é dado de saúde sensível. |
| Origem | Theo, kickoff de 2026-09-15 |
| Status | approved |
| Desde | v1.0.0 |

## BR-09 — Exportação nunca bloqueada

A exportação dos dados da clínica nunca é bloqueada por plano, inadimplência ou cancelamento.

| Atributo | Valor |
| --- | --- |
| Justificativa | A portabilidade é garantia do produto, não recurso de plano. |
| Origem | Theo, kickoff de 2026-09-15 |
| Status | approved |
| Desde | v1.0.0 |

## BR-10 — O paciente não opera o sistema

O paciente não tem conta nem acesso ao sistema; ele interage por WhatsApp, por e-mail e pelos documentos que recebe.

| Atributo | Valor |
| --- | --- |
| Justificativa | App de clínica única não tem adoção; o produto não é marketplace. |
| Origem | Theo, kickoff de 2026-09-15 |
| Status | approved |
| Desde | v1.0.0 |

# 0013. Assinatura digital em duas camadas, por provedor existente

- Status: accepted
- Date: 2026-10-03
- Registra: a decisão tomada em 2026-09-15, que morava no `docs/requirements.md` e saiu de lá quando
  ele virou o SRS em `docs/requirements/` (#139)

## Context

Eliminar o papel do prontuário exige assinatura com validade jurídica, e a exigência muda conforme
quem assina e o que é assinado. As regras estão no SRS, como `BR-01` a `BR-06` de
`docs/requirements/business-rules.md`:

- Lei 14.063/2020, art. 13 — atestado e receita de controle especial exigem assinatura qualificada.
- Lei 14.063/2020, art. 14 — os demais documentos do profissional de saúde valem com avançada ou
  qualificada.
- Lei 14.063/2020, art. 5º, §5º — em conflito entre normas, prevalece a qualificada.
- Resolução CFO 91/2009 — dispensa o papel só com prontuário NGS2, que exige ICP-Brasil.
- Resolução CFO 278/2025, art. 8º, §1º — o prontuário leva a assinatura do dentista e a do paciente
  ou responsável legal.

Eliminar todo o papel não é viável já: depende de o dentista ter certificado ICP-Brasil.

## Decision

O Clinicore não constrói assinatura própria. Usa provedores com validade jurídica, preferindo os
gratuitos ou com plano gratuito que cubra a fase de teste. São duas camadas:

1. **Dentista com certificado** — assinatura qualificada ICP-Brasil com o certificado em nuvem do
   próprio dentista (Bird ID, VIDaaS, SafeID), pela API do PSC. Uma autorização no app do celular abre
   uma sessão de assinatura de até 7 dias, o limite do ITI para pessoa física, e o sistema assina tudo
   nesse período sem o dentista sair do Clinicore. Cobre todos os documentos. O certificado é custo do
   dentista.
2. **Dentista sem certificado** — assinatura avançada dentro do sistema para prontuário, receita
   comum, laudo e pedido de exame. Atestado e receita de controle especial ficam sem assinatura
   eletrônica até haver certificado; o documento é impresso e assinado à mão.
3. **Paciente** — assinatura eletrônica avançada por plataforma de terceiros, num dispositivo da
   clínica ou por link enviado ao e-mail do paciente. Na fase de teste, a plataforma é a
   **Autentique**: plano gratuito de 10 documentos por mês e API com sandbox.
4. **Papel como saída** — qualquer documento pode ser impresso.

## Consequences

- `FR-CLN-07`, `FR-CLN-08` e `FR-CLN-09` descrevem a capacidade; esta ADR diz com quem ela é entregue.
- Trocar de provedor muda esta ADR, não o SRS.
- O dentista sem certificado continua com papel em atestado e receita de controle especial.
- Seguem em aberto, em `docs/requirements/open-questions.md`: o parecer jurídico sobre o alcance da
  assinatura avançada (`OQ-04`) e o custo da integração com PSC e da API da Autentique em produção
  (`OQ-05`).

## Alternatives considered

- **ZapSign** — plano gratuito de 3 documentos por mês e sem API; a API começa no plano pago.
- **SignDocs** — plano gratuito de 5 documentos por mês com ICP-Brasil, mas a API só existe no plano
  Enterprise, sob consulta.
- **Assinatura gov.br** — a API é restrita a órgãos públicos; o fluxo manual de baixar, assinar fora e
  reenviar é o de maior atrito; não serve para atestado nem para receita de controle especial; e a
  aceitação entre particulares é contestada.

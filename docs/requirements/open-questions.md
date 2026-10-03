# Perguntas em aberto

O que ainda não foi decidido. Uma pergunta resolvida sai daqui e dá lugar a um link para o que a
resolveu.

## OQ-01 — Nome comercial

Clinicore é codinome interno: está bloqueado para uso comercial, por já existir como marca e por ser
próximo do concorrente Clinicorp. O produto é renomeado antes de ser vendido.

Restrições que o nome precisa cumprir:

- soar bem na pronúncia brasileira;
- não existir em software nem em saúde;
- não ter registro vivo no INPI nas classes 42 e 44.

O que já foi recusado, por tipo: variação mecânica de um nome de que o Theo gostou (Prontia, Oryvel),
palavra portuguesa antiquada (Mola Mestra, Colmeia), palavra inglesa que soa mal na pronúncia
brasileira (Crux, Kernel, Shift), nome literal de odontologia (Odoncore, Molar) e nome com sentido
cotidiano ruim (Granada). Referência de bom nome abstrato: Notion. Oryn falha no INPI, com dois
registros na classe 44.

## OQ-02 — Formato de saída do Simples Dental

O que a exportação deles entrega de fato — arquivos, campos, lacunas — e se há outra via de extração.
Em análise pelo Theo. Bloqueia [FR-PRT-01](functional/portability.md#fr-prt-01--importação-do-simples-dental).

## OQ-03 — Importação de outros legados

Clinicorp e softwares desktop antigos: quando e quais.

## OQ-04 — Assinatura digital: parecer jurídico

(a) A assinatura avançada do art. 14 da Lei 14.063 basta para eliminar o papel do prontuário, apesar
da Resolução CFO 91/2009 e do art. 5º, §5º? (b) A assinatura avançada basta para os termos do
paciente?

## OQ-05 — Assinatura digital: custos e requisitos

(a) Se a integração com PSC de certificado em nuvem tem custo ou credenciamento para o integrador.
(b) O preço da API do provedor de assinatura do paciente em produção. (c) Quais requisitos do NGS2,
além do certificado ICP-Brasil, o sistema precisa cumprir.

## OQ-06 — Operadoras do primeiro corte TISS

Quais operadoras odontológicas entram no primeiro corte de
[FR-INS-02](functional/insurance.md#fr-ins-02--guias-e-lotes-em-arquivo-tiss). Não bloqueia o go-live.

## OQ-07 — IA: áudio da consulta

Onde os áudios são processados e por quanto tempo ficam retidos. Decidido na spec da IA clínica.

## OQ-08 — Modelo comercial do SaaS

Planos, preço e cobrança por clínica ou por dentista. Adiado para depois do sistema pronto. Bloqueia
[FR-SITE-04](functional/site.md#fr-site-04--preços-e-planos).

## OQ-09 — Prazo de guarda do prontuário

Por quanto tempo o prontuário e seus documentos são guardados, e o que acontece com eles quando o
cliente cancela.

## OQ-10 — Direitos do titular na LGPD

Como o paciente exerce acesso, correção e eliminação dos próprios dados, se ele não opera o sistema.

## OQ-11 — Backup e recuperação

Quanto dado a clínica aceita perder numa falha e em quanto tempo o sistema precisa voltar.

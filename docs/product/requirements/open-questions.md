# Perguntas em aberto

O que ainda não foi decidido. Uma pergunta resolvida sai daqui e dá lugar a um link para o que a
resolveu.

## OQ-01 — Nome comercial

Clinicore é codinome interno e não pode virar marca:

- a marca "Clinicore" foi depositada pela FAN Marketing Digital na classe 42 do INPI e está em prazo de
  oposição desde 2026-08-20;
- o domínio `clinicore.com.br` pertence a uma clínica médica;
- o nome é próximo do concorrente Clinicorp.

O produto é construído como Clinicore e renomeado antes de ser vendido. A escolha do nome não bloqueia
o design system nem as telas.

Restrições que o nome precisa cumprir:

- soar bem na pronúncia brasileira, para quem não fala inglês;
- fazer sentido à primeira leitura e soar como software de clínica;
- não ser nome de pessoa;
- não ter marca viva nem depositada no INPI nas classes 09, 10, 42 e 44;
- ter o `.com.br` exato livre: um domínio com qualquer dono desclassifica o nome.

O foco em odontologia vale para o nome, como vale para o produto.

A marca gráfica já está aprovada, a rota Arcada, e não depende do nome: o arco dentário visto de cima,
desenhado como o arco modernista.

O que já foi recusado, por tipo:

- variação mecânica de um nome de que o Theo gostou (Prontia, Oryvel, Oclusia);
- palavra portuguesa antiquada ou de arquitetura (Mola Mestra, Colmeia, Arquivolta, Peristilo, Dovela);
- palavra inglesa que soa mal na pronúncia brasileira (Crux, Kernel, Shift);
- palavra japonesa (Irori, Teire, Mamori, Musubi);
- nome mítico ou histórico (Hesira);
- nome descritivo em português (Dental em Dia, Clínica Inteira) e palavra cotidiana sem sentido de
  software (Fôlego, Clinamen, Selar);
- nome literal de odontologia (Odoncore, Molar) e nome com sentido cotidiano ruim (Granada).

A direção que mais agradou é um termo técnico que o dentista reconhece e que tem um segundo sentido:
Oclusa, que caiu porque o `.com.br` tem dono.

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

Onde os áudios são processados e por quanto tempo ficam retidos. Será decidido na spec da IA clínica.

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

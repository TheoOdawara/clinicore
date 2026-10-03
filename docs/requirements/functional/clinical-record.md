# Prontuário clínico — `CLN`

O registro odontológico do paciente e os documentos emitidos e assinados a partir dele.

## FR-CLN-01 — Anamnese

> Como dentista, quero registrar a anamnese do paciente, para que o histórico de saúde esteja disponível a cada atendimento.

O sistema deve manter a anamnese do paciente.

| Atributo | Valor |
| --- | --- |
| Justificativa | A anamnese compõe o prontuário do paciente. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-01.1** — Dada uma anamnese preenchida, quando o dentista abre o prontuário, então a vê com a data do preenchimento.

## FR-CLN-02 — Odontograma

> Como dentista, quero registrar a situação de cada dente, para que eu veja a boca do paciente de uma vez.

O sistema deve manter o odontograma do paciente, com a situação registrada por dente.

| Atributo | Valor |
| --- | --- |
| Justificativa | O odontograma mostra a situação da boca do paciente de uma vez. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-02.1** — Dado um dente marcado com uma situação, quando o odontograma é reaberto, então o dente mostra a situação registrada.

## FR-CLN-03 — Plano de tratamento com orçamento

> Como dentista, quero montar o plano de tratamento e gerar o orçamento a partir dele, para que o que é proposto ao paciente seja o que é cobrado.

O sistema deve manter o plano de tratamento do paciente e gerar o orçamento vinculado a ele.

| Atributo | Valor |
| --- | --- |
| Justificativa | O orçamento nasce do plano, para que os dois não divirjam. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-03.1** — Dado um plano com procedimentos, quando o orçamento é gerado, então traz os mesmos procedimentos e fica vinculado ao plano.

## FR-CLN-04 — Evolução clínica por atendimento

> Como dentista, quero registrar o que foi feito em cada atendimento, para que o tratamento tenha história.

O sistema deve registrar a evolução clínica de cada atendimento, vinculada ao paciente e à consulta.

| Atributo | Valor |
| --- | --- |
| Justificativa | A evolução registra o que foi feito em cada atendimento. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-04.1** — Dado um atendimento concluído, quando o dentista registra a evolução, então ela aparece no prontuário com a data e o autor.

## FR-CLN-05 — Emissão de documentos

> Como dentista, quero emitir termo de consentimento, atestado e receita, para que eu não redija cada documento à mão.

O sistema deve emitir termo de consentimento, atestado e receita a partir dos dados do paciente.

| Atributo | Valor |
| --- | --- |
| Justificativa | O documento emitido no sistema fica no prontuário. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-05.1** — Dado um paciente, quando o dentista emite um atestado, então o documento fica vinculado ao paciente.

## FR-CLN-06 — Impressão de documentos

> Como recepção, quero imprimir qualquer documento do paciente, para que o papel exista quando a assinatura eletrônica não alcança.

O sistema deve gerar a versão imprimível de qualquer documento do paciente.

| Atributo | Valor |
| --- | --- |
| Justificativa | A lei ainda exige papel em parte dos casos (BR-05, BR-06). |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-06.1** — Dado qualquer documento do paciente, quando alguém pede a impressão, então o sistema gera a versão imprimível.

## FR-CLN-07 — Assinatura qualificada do dentista

> Como dentista, quero assinar com meu certificado em nuvem sem aprovar cada documento no celular, para que eu assine tudo sem sair do sistema.

O sistema deve assinar documentos com o certificado ICP-Brasil em nuvem do dentista, dentro de uma sessão de assinatura autorizada por ele.

| Atributo | Valor |
| --- | --- |
| Justificativa | Só a assinatura qualificada cobre atestado e receita de controle especial (BR-01). |
| Origem | Theo, kickoff de 2026-09-15 · [ADR 0013](../../decisions/0013-assinatura-digital-em-duas-camadas.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-07.1** — Dado um dentista que autorizou a sessão de assinatura, quando assina documentos dentro do período autorizado, então não aprova cada um no celular.
- **FR-CLN-07.2** — Dada uma primeira consulta com dentista que tem certificado em nuvem, quando ela termina, então anamnese, odontograma, plano, evolução e termos assinados existem só no sistema.

## FR-CLN-08 — Assinatura avançada do dentista

> Como dentista, quero assinar dentro do sistema sem ter certificado, para que o prontuário não dependa de eu comprar um certificado.

O sistema deve oferecer ao dentista sem certificado a assinatura eletrônica avançada nos documentos que a lei admite.

| Atributo | Valor |
| --- | --- |
| Justificativa | O certificado é custo do dentista; sem ele o sistema ainda elimina parte do papel (BR-02). |
| Origem | Theo, kickoff de 2026-09-15 · [ADR 0013](../../decisions/0013-assinatura-digital-em-duas-camadas.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-08.1** — Dado um dentista sem certificado, quando registra uma evolução, então a assina com a assinatura avançada.
- **FR-CLN-08.2** — Dado um dentista sem certificado, quando emite um atestado, então o sistema não oferece assinatura eletrônica e oferece a impressão.

## FR-CLN-09 — Assinatura do paciente

> Como recepção, quero colher a assinatura do paciente num dispositivo da clínica ou por link enviado ao e-mail dele, para que o termo volte assinado sem papel.

O sistema deve colher a assinatura eletrônica do paciente ou do responsável legal num dispositivo da clínica ou por link enviado por e-mail.

| Atributo | Valor |
| --- | --- |
| Justificativa | O prontuário leva a assinatura do paciente (BR-04). |
| Origem | Theo, kickoff de 2026-09-15 · [ADR 0013](../../decisions/0013-assinatura-digital-em-duas-camadas.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-09.1** — Dado um termo pendente, quando a recepção escolhe o dispositivo da clínica ou o e-mail, então o paciente assina pelo canal escolhido.
- **FR-CLN-09.2** — Dado um termo assinado, quando a assinatura é concluída, então o documento assinado fica vinculado ao paciente.

## FR-CLN-10 — Imagens e exames anexados

> Como dentista, quero anexar radiografias, fotos e exames ao paciente, para que eu os consulte durante o atendimento.

O sistema deve armazenar imagens e arquivos de exame vinculados ao paciente.

| Atributo | Valor |
| --- | --- |
| Justificativa | Imagens e exames ficam junto do paciente. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **FR-CLN-10.1** — Dado um exame anexado, quando o dentista abre o prontuário, então o arquivo está disponível para visualização.

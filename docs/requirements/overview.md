# Visão geral

Quem usa o produto, onde ele roda, o que o limita e do que ele depende.

## Stakeholders

| Papel | Usa para |
| --- | --- |
| Recepção | Agenda, cadastro, confirmação e recebimento |
| Dentista | A própria agenda, o prontuário, o plano de tratamento e a estética |
| Dentista autônomo | Tudo que o dentista usa, como cliente próprio, sem clínica por trás |
| Gestor da clínica | Financeiro, indicadores, equipe e configuração da clínica |
| Dono de rede | Visão consolidada de várias clínicas, cada uma com gestor próprio |
| Responsável por estoque e laboratório | Movimentação de estoque e pedidos ao laboratório protético |
| Visitante | Conhecer o produto pela landing, antes de ser cliente |
| Paciente | Não opera o sistema ([BR-10](business-rules.md#br-10--o-paciente-não-opera-o-sistema)); recebe mensagens no WhatsApp, e-mails e documentos |

**Decisor do produto:** Theo. **Clínicas piloto:** duas clínicas odontológicas da família, cada uma
operando como cliente independente.

## Ambiente de operação

| Superfície | Endereço | Quem usa | Conexão |
| --- | --- | --- | --- |
| Sistema web | `app.clinicore.com.br` | usuário autenticado | exige conexão |
| App nativo de iOS e de Android | instalado no aparelho | usuário autenticado | funciona sem conexão |
| API | `api.clinicore.com.br` | o web e o app | — |
| Landing | `clinicore.com.br` | visitante | exige conexão |

O paciente é alcançado por WhatsApp e por e-mail, nunca por uma tela do sistema.

## Restrições

- **Só odontologia.** O produto não atende outra especialidade.
- **WhatsApp pela Twilio.** Confirmação, lembrete e o agente de atendimento usam a Twilio como canal.
- **Assinatura digital por provedor existente.** O produto não constrói assinatura própria; a decisão
  de provedores está na [ADR 0013](../decisions/0013-assinatura-digital-em-duas-camadas.md).
- **O certificado ICP-Brasil é custo do dentista**, não do Clinicore.
- **Convênio só por arquivo TISS**, sem integração direta com a operadora.
- **Laboratório protético sem integração**: tudo é lançado à mão.
- **A stack** é a do [`AGENTS.md`](../../AGENTS.md) da raiz e das ADRs em [`decisions/`](../decisions/).

## Premissas e dependências

- A importação dos dados da clínica piloto é feita à mão pelo Theo, no onboarding.
- Eliminar todo o papel depende de o dentista ter certificado em nuvem; sem ele, parte dos documentos
  é impressa ([BR-01](business-rules.md#br-01--assinatura-qualificada-para-atestado-e-receita-de-controle-especial),
  [BR-06](business-rules.md#br-06--impressão-sempre-disponível)).
- O alcance da assinatura avançada depende de parecer jurídico
  ([OQ-04](open-questions.md#oq-04--assinatura-digital-parecer-jurídico)).
- A página de preços depende do modelo comercial
  ([OQ-08](open-questions.md#oq-08--modelo-comercial-do-saas)).

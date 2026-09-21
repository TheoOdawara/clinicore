# Clinicore — Requisitos do produto

> Codinome interno. Nome comercial pendente (ver Perguntas em aberto).

## 1. Problema
Clínicas odontológicas operam com sistemas legados (Clinicorp, Simples Dental, softwares desktop antigos)
ou papel/planilha. Nenhum cobre a clínica de ponta a ponta:
- **Não é E2E** — agenda, clínico, financeiro, convênio e atendimento ficam espalhados entre ferramentas.
- **Estética ignorada** — harmonização orofacial é forte na odontologia hoje e quase não tem suporte.
- **UX ruim** — telas lentas, confusas, pensadas para desktop.
- **Aprisionamento de dados** — sair do sistema é inviável: a exportação é incompleta ou vem errada
  (caso real: Simples Dental).

O Clinicore existe para eliminar o papel da clínica até onde a regulação permite, cobrir o fluxo inteiro
num só lugar, e devolver ao cliente o controle dos próprios dados.

## 2. Usuários e stakeholders
| Papel | Usa para |
|---|---|
| Recepção / secretária | Agenda, cadastro, confirmação, recebimento |
| Dentista | Própria agenda, prontuário, plano de tratamento, estética |
| Dentista autônomo | Tudo acima, como cliente próprio, sem clínica por trás |
| Gestor da clínica | Financeiro, indicadores, equipe, configuração da clínica |
| Dono de rede | Visão consolidada de várias clínicas, cada uma com gestor próprio |
| Responsável por estoque / laboratório | Movimentação de estoque e pedidos ao laboratório protético |
| Paciente | **Não opera o sistema.** Interage só via WhatsApp e documentos recebidos |

Decisor do produto: Theo. Clínicas piloto: clínica da mãe e clínica dos tios (clientes separados).

## 3. Escopo

### No MVP
- Multi-tenant com suporte a redes de clínicas e a dentistas autônomos
- Pacientes
- Agenda com confirmação e lembrete
- Prontuário clínico odontológico (papel só onde a assinatura eletrônica não alcança)
- Prontuário estético (harmonização orofacial)
- Financeiro da clínica, com emissão de NFS-e
- Estoque
- Laboratório protético (controle manual)
- Convênios / TISS (arquivo)
- Portabilidade: importar do Simples Dental e exportar tudo
- Diferenciais: IA no atendimento, IA clínica, indicadores de gestão

### Go-live da clínica piloto
O corte mínimo para a primeira clínica piloto sair do Simples Dental:
1. Organização e acesso
2. Pacientes
3. Agenda, com confirmação e lembrete por WhatsApp
4. Prontuário clínico e estético
5. Financeiro, com NFS-e
6. Estoque
7. Laboratório protético
8. Indicadores de gestão

Portabilidade fica fora do go-live: a importação da clínica piloto é feita manualmente, no onboarding
conduzido pelo Theo. Convênios/TISS, portabilidade, IA no atendimento e IA clínica entram depois do
go-live.

### Fora do escopo
- **App do paciente / autoagendamento** — app de clínica única não tem adoção; só faria sentido como
  marketplace multi-clínica, que não é este produto.
- Marketplace de agendamento entre clínicas.
- Especialidades não odontológicas (o produto é só odontologia).
- Integração direta com operadoras de convênio (só arquivo TISS).
- Integração com laboratórios protéticos (controle é manual).
- Cobrança do SaaS (planos, preço, assinatura) — vem depois do sistema.

## 4. Requisitos funcionais

### 4.1 Organização e acesso
- Multi-tenant: cada cliente é isolado dos demais.
- Um cliente pode ser uma clínica única, uma **rede** de clínicas (cada clínica com gestor(es) próprio(s))
  ou um **dentista autônomo**.
- O cliente pode ser pessoa física ou jurídica.
- Papéis com permissões distintas (recepção, dentista, gestor da clínica, dono de rede, responsável por
  estoque/laboratório).
- Dono de rede vê dados consolidados; gestor vê só a própria clínica.

### 4.2 Pacientes
- Cadastro completo, com responsável legal para menores.
- Histórico único do paciente: clínico, estético, financeiro, comunicação.

### 4.3 Agenda
- Agenda por dentista e por cadeira/sala.
- Confirmação e lembrete automáticos por WhatsApp.
- Registro de falta, remarcação, encaixe, lista de espera.

### 4.4 Prontuário clínico
- Anamnese.
- Odontograma.
- Plano de tratamento com orçamento vinculado.
- Evolução clínica por atendimento.
- Documentos (termos de consentimento, atestados, receitas) emitidos e assinados digitalmente.
- Todo documento pode ser impresso. O que não puder ser assinado eletronicamente é impresso e assinado
  à mão.
- Assinatura do paciente em qualquer dispositivo da clínica (tablet, celular, computador) ou por link
  enviado ao e-mail do paciente.
- Imagens e exames anexados ao paciente.

### 4.5 Prontuário estético
- Procedimentos de harmonização orofacial (toxina, preenchedor, bioestimulador, etc.).
- Mapa facial dos pontos/áreas aplicados.
- Rastreabilidade do produto usado (marca, lote, validade, quantidade).
- Fotos padronizadas antes/depois com comparação.
- Termos de consentimento específicos de estética.

### 4.6 Financeiro
- Orçamento → aceite → cobrança.
- Recebimentos, parcelamento, formas de pagamento.
- Caixa, contas a pagar e a receber.
- Repasse/comissão de dentista.
- Inadimplência.
- Emissão de NFS-e.

### 4.7 Convênios / TISS
- Cadastro de operadoras e tabelas.
- Geração de guias e lotes em arquivo no padrão TISS para as principais operadoras odontológicas.
- Acompanhamento de glosa e recurso, registrado manualmente.

### 4.8 Portabilidade
- **Importação** do Simples Dental: pacientes, agenda, prontuário, financeiro.
- Relatório de importação: o que entrou, o que falhou e por quê — nada some em silêncio.
- **Exportação completa** de todos os dados da clínica, a qualquer momento, em formato aberto e
  documentado, por self-service.

### 4.9 IA no atendimento
- Agente no WhatsApp que confirma, remarca e agenda consultas.
- Reativação de pacientes inativos e de orçamentos não aprovados.
- Passagem para humano quando o agente não resolve.

### 4.10 IA clínica
- Transcrição da consulta gerando rascunho de evolução no prontuário.
- O dentista revisa e aprova; nada entra no prontuário sem aprovação humana.

### 4.11 Indicadores de gestão
- Funil orçamento → aceite.
- Ocupação/ociosidade de cadeira e de dentista.
- Taxa de faltas.
- Inadimplência e faturamento.
- Pacientes inativos / reativados.
- Visão por clínica e consolidada por rede.

### 4.12 Estoque
- Cadastro de materiais e produtos (incluindo os estéticos).
- Entradas, saídas e saldo, com lote e validade.
- Baixa do produto estético usado no procedimento.
- Alerta de estoque baixo e de validade próxima.

### 4.13 Laboratório protético
- Cadastro de laboratórios.
- Pedido vinculado ao paciente e ao tratamento: envio, prazo, retorno, status.
- Custo do pedido refletido no financeiro.
- Tudo registrado manualmente no sistema, sem integração com o laboratório.

## 5. Requisitos não funcionais
- **Mobile-first** — toda tela é desenhada na largura de celular antes do desktop. UX simples é o
  principal argumento contra o legado.
- **Plataforma** — web responsiva e app nativo iOS e Android, os dois no MVP e no go-live da clínica
  piloto, com as mesmas telas. O web exige conexão; o app funciona sem ela
  (`docs/decisions/0007-app-nativo-em-flutter-com-offline.md`).
- **Offline no app** — sem internet, o app mostra e grava como se estivesse online; o que foi gravado
  espera numa fila e sobe quando a conexão volta. Uma escrita feita sobre dado que mudou no servidor é
  recusada e fica em pendência para a pessoa refazer — nada é sobrescrito em silêncio. O dado de saúde
  guardado no aparelho é cifrado, abre com biometria ou PIN, e é apagado após 72 horas sem sincronizar,
  no logout e na revogação da sessão; a fila pendente não é apagada.
- **Segurança e LGPD** — dado de saúde é dado sensível: isolamento entre tenants, criptografia em
  trânsito e repouso, trilha de auditoria de acesso ao prontuário, menor privilégio por papel.
- **Validade legal do prontuário eletrônico** — eliminar papel exige conformidade com as normas do CFO e
  da legislação de prontuário digital, incluindo assinatura digital com validade jurídica.
- **Assinatura digital por serviço existente** — não construir assinatura própria; usar provedores com
  validade jurídica, preferindo gratuitos ou com plano gratuito que cubra a fase de teste. São duas
  camadas, porque a exigência legal muda conforme quem assina. Base legal:
  - Lei 14.063/2020, art. 13 — atestado e receita de controle especial exigem assinatura **qualificada**
    (ICP-Brasil).
  - Lei 14.063/2020, art. 14 — os demais documentos do profissional de saúde valem com assinatura
    **avançada** ou qualificada.
  - Lei 14.063/2020, art. 5º, §5º — em conflito entre normas, prevalece a qualificada.
  - Resolução CFO 91/2009 — dispensa o papel só com prontuário NGS2, que exige ICP-Brasil.
  - Resolução CFO 278/2025, art. 8º, §1º — o prontuário leva assinatura do dentista e do paciente ou
    responsável legal.

  Decisão:
  1. **Dentista com certificado** — assinatura qualificada ICP-Brasil com certificado em nuvem do próprio
     dentista (Bird ID, VIDaaS, SafeID etc.) via API de PSC. Uma autorização no app do celular abre uma
     sessão de assinatura de até 7 dias (limite do ITI para pessoa física), e o sistema assina tudo nesse
     período sem o dentista sair do Clinicore. Cobre todos os documentos. O certificado é custo do
     dentista, não do Clinicore.
  2. **Dentista sem certificado** — assinatura avançada dentro do sistema para prontuário, receita comum,
     laudo e pedido de exame. Atestado e receita de controle especial ficam bloqueados para assinatura
     eletrônica até haver certificado; nesse caso o documento é impresso e assinado à mão.
  3. **Paciente** — assinatura eletrônica avançada por plataforma de terceiros, feita em qualquer
     dispositivo da clínica (tablet, celular, computador) ou por link enviado ao e-mail do paciente.
     Candidata para a fase de teste: **Autentique** (plano gratuito de 10 documentos/mês, API com
     sandbox). Alternativas avaliadas: ZapSign (grátis 3/mês, sem API; API a partir do plano pago),
     SignDocs (grátis 5/mês com ICP-Brasil, mas API só no plano Enterprise sob consulta).
  4. **Papel como saída** — qualquer documento pode ser impresso; o que não puder ser assinado
     eletronicamente é assinado à mão.

  Descartada: assinatura gov.br — a API é restrita a órgãos públicos, o fluxo manual (baixar, assinar
  fora, reenviar) é o de maior atrito, não serve para atestado nem receita controlada, e a aceitação entre
  particulares é contestada.
- **WhatsApp via Twilio** — confirmação, lembrete e IA no atendimento usam a Twilio como canal.
- **Consentimento para IA** — gravação/transcrição de consulta só com consentimento do paciente registrado.
- **Portabilidade como garantia** — exportação é completa, fiel e sempre disponível; nunca é bloqueada
  por plano, inadimplência ou cancelamento.
- **Disponibilidade** — agenda e prontuário são críticos na operação diária da clínica.
- **Acessibilidade** — contraste, navegação por teclado, leitores de tela.
- **Idioma** — interface em PT-BR.

## 6. Critérios de aceite (nível de capacidade)
- **Tenancy** — dado um usuário da clínica A, quando ele busca pacientes, então nunca vê dado da clínica B.
- **Rede** — dado um dono de rede com 3 clínicas, quando abre os indicadores, então vê o consolidado e
  cada clínica; dado o gestor de uma delas, então vê só a dele.
- **Dentista autônomo** — dado um dentista pessoa física sem clínica, quando se cadastra, então opera
  agenda, prontuário e financeiro como cliente próprio.
- **Agenda** — dado um agendamento, quando chega a janela de lembrete, então o paciente recebe a
  confirmação no WhatsApp e a resposta atualiza a agenda.
- **Sem papel** — dada uma primeira consulta com dentista que tem certificado em nuvem, quando ela
  termina, então anamnese, odontograma, plano, evolução e termos assinados existem só no sistema, com
  validade legal.
- **Sessão de assinatura** — dado um dentista que autorizou o certificado em nuvem, quando assina
  documentos dentro do período autorizado, então não precisa aprovar cada um no celular.
- **Sem certificado** — dado um dentista sem certificado, quando emite um atestado, então o sistema não
  oferece assinatura eletrônica e oferece a impressão para assinatura à mão.
- **Assinatura do paciente** — dado um termo pendente, quando a recepção escolhe assinar num tablet da
  clínica ou enviar por e-mail, então o paciente assina pelo canal escolhido e o termo assinado volta
  vinculado ao paciente.
- **Impressão** — dado qualquer documento do paciente, quando alguém pede para imprimir, então o sistema
  gera a versão imprimível.
- **Estética** — dado um procedimento de toxina, quando registrado, então fica o mapa facial, o lote do
  produto e as fotos antes/depois vinculados ao paciente, e o produto sai do estoque.
- **Financeiro** — dado um orçamento aceito e parcelado, quando uma parcela vence sem pagamento, então
  aparece na inadimplência e no indicador.
- **NFS-e** — dado um recebimento, quando o gestor emite a nota, então a NFS-e é gerada e fica vinculada
  ao recebimento.
- **Estoque** — dado um produto com saldo abaixo do mínimo ou validade próxima, quando o responsável abre
  o estoque, então vê o alerta.
- **Laboratório** — dado um pedido enviado ao laboratório, quando o prazo passa sem retorno, então o
  pedido aparece como atrasado.
- **TISS** — dado um atendimento por convênio, quando faturado, então gera o arquivo TISS aceito pela
  operadora e a glosa pode ser registrada e acompanhada.
- **Importação** — dada uma exportação do Simples Dental, quando importada, então todo registro entra ou
  aparece no relatório com o motivo da falha.
- **Exportação** — dado um gestor, quando pede a exportação, então recebe todos os dados da clínica em
  formato aberto e documentado, sem depender de suporte.
- **IA atendimento** — dado um paciente que pede remarcação no WhatsApp, quando o agente tem horário,
  então remarca; quando não resolve, então transfere para a recepção.
- **IA clínica** — dada uma consulta gravada com consentimento, quando termina, então o dentista recebe
  um rascunho de evolução que só entra no prontuário após aprovar.
- **Offline** — dado um dentista com o app sincronizado, quando perde a internet, então vê a agenda e os
  pacientes dela e registra o atendimento; quando a conexão volta, então o registro chega à API sem ação
  dele.
- **Conflito offline** — dada uma consulta cancelada na recepção enquanto o dentista estava offline,
  quando o registro dele sobe, então a API o recusa e o app o mostra em pendência.
- **Expiração offline** — dado um app sem sincronizar há 72 horas, quando é aberto, então não mostra dado
  de saúde e mantém a fila pendente.
- **Indicadores** — dado um mês de operação, quando o gestor abre o painel, então vê funil de orçamento,
  ocupação, faltas e inadimplência.

## 7. Perguntas em aberto
1. **Nome comercial** — Clinicore é bloqueado para uso comercial; renomear antes de vender.
2. **Formato de saída do Simples Dental** — o que a exportação deles entrega de fato (arquivos, campos,
   lacunas) e se há outra via de extração. Em análise pelo Theo.
3. **Importação de outros legados** (Clinicorp, desktop antigo) — quando e quais.
4. **Assinatura digital — parecer jurídico** — (a) a assinatura avançada do art. 14 da Lei 14.063 basta
   para eliminar o papel do prontuário, apesar da Resolução CFO 91/2009 e do art. 5º, §5º? (b) a
   assinatura avançada basta para os termos do paciente?
5. **Assinatura digital — custos e requisitos** — (a) se a integração com PSC de certificado em nuvem
   (Bird ID, VIDaaS) tem custo ou credenciamento para o integrador; (b) preço da API do Autentique em
   produção; (c) quais requisitos do NGS2, além do certificado ICP-Brasil, o sistema precisa cumprir.
6. **Operadoras TISS** — quais são as "principais operadoras" do primeiro corte. Adiado; não bloqueia o
   go-live.
7. **IA — áudio** — onde os áudios são processados e por quanto tempo ficam retidos. Decidir na spec da
   IA clínica.
8. **Modelo comercial do SaaS** — planos, preço e cobrança por clínica/dentista. Adiado para depois do
   sistema pronto.

## Evolution log
- **2026-09-15** — Kickoff. Problema, usuários, escopo do MVP (agenda, prontuário clínico e estético,
  financeiro, TISS, portabilidade, IA no atendimento, IA clínica, indicadores), multi-tenant com redes,
  app do paciente fora de escopo.
- **2026-09-15** — Corte do go-live da clínica piloto definido (acesso, pacientes, agenda, prontuário
  clínico e estético, financeiro, indicadores); portabilidade sai do go-live e a importação inicial vira
  manual. Entram no MVP: NFS-e, estoque e laboratório protético manual. Decididos: assinatura digital por
  provedor existente, WhatsApp via Twilio, TISS só por arquivo, cliente pode ser clínica ou dentista
  autônomo, PF ou PJ. Cobrança do SaaS adiada.
- **2026-09-15** — Go-live ampliado com NFS-e, estoque, laboratório protético e confirmação por
  WhatsApp. Assinatura digital pesquisada e dividida em duas camadas: qualificada ICP-Brasil do dentista
  via certificado em nuvem (exigência CFO 91/2009 e Lei 14.063/2020) e eletrônica do paciente via
  Autentique na fase de teste. Operadoras TISS adiadas.
- **2026-09-15** — Assinatura revista após pesquisa da Lei 14.063 (arts. 5º, 13 e 14) e das Resoluções
  CFO 91/2009 e 278/2025. Eliminar 100% do papel não é viável já: dentista com certificado em nuvem
  assina tudo com sessão de até 7 dias; sem certificado, assinatura avançada no que a lei permite e
  impressão no resto; paciente assina em qualquer dispositivo da clínica ou por e-mail; impressão sempre
  disponível. gov.br descartado. Parecer jurídico sobre assinatura avançada vira pergunta em aberto.
- **2026-09-21** — App nativo iOS e Android entra no MVP e no go-live da piloto, em Flutter, com paridade
  de telas com o web. Offline completo no app, com fila de escrita, recusa por versão e pendência, dado
  cifrado apagado após 72 horas sem sincronizar. O web segue exigindo conexão. Decisão na ADR 0007.

# Riscos e dívida técnica

O que se sabe que está frágil ou em aberto, a consequência e a issue que acompanha. O estado de cada
issue mora no GitHub, nunca aqui.

## Riscos de arquitetura

| Risco | Consequência | Acompanha |
| --- | --- | --- |
| Não há hospedagem, Dockerfile nem deploy | nada do que existe foi exercitado fora da máquina de desenvolvimento e do CI | [deployment.md](deployment.md) |
| Backup e recuperação sem decisão | a perda aceitável de dado e o tempo de volta não têm número | [OQ-11](../requirements/open-questions.md#oq-11--backup-e-recuperação) |
| A purga roda dentro de cada réplica da API | réplicas repetem o `DELETE`; inofensivo, mas é trabalho que pertence ao worker | #71 |
| O offline ainda não tem spec | versão, UUID do cliente e idempotência precisam nascer com a primeira rota de escrita do produto, ou viram migração | [ADR 0007](../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| O web replica o contrato da API à mão, em Zod | uma mudança na API que o web não acompanha só aparece em runtime | [ADR 0007](../decisions/0007-app-nativo-em-flutter-com-offline.md), item 2 |
| `main` e `develop` protegidas só por convenção | um push direto não é barrado pelo GitHub | [`AGENTS.md`](../../AGENTS.md) → Branches |
| Parecer jurídico e custo da assinatura digital em aberto | o alcance da assinatura avançada pode mudar o fluxo do prontuário | [OQ-04](../requirements/open-questions.md#oq-04--assinatura-digital-parecer-jurídico), [OQ-05](../requirements/open-questions.md#oq-05--assinatura-digital-custos-e-requisitos) |
| Prazo de guarda do prontuário e direitos do titular sem decisão | o modelo de dados clínico nasce sem saber quando e como apaga | [OQ-09](../requirements/open-questions.md#oq-09--prazo-de-guarda-do-prontuário), [OQ-10](../requirements/open-questions.md#oq-10--direitos-do-titular-na-lgpd) |

## Dívida de segurança

Issues com a label `security-debt`, todas do épico #124.

| O que | Issue |
| --- | --- |
| Cadastro duplicado de um e-mail não verificado permite pré-sequestro da conta | #130 |
| Hash Argon2 sem teto de concorrência | #129 |
| Rastreio do IP do cliente no limite de requisições | #131 |
| Caminhos de abuso de baixa severidade no cadastro | #136 |

## Dívida técnica

Issues com a label `tech-debt`, todas do épico #124.

| O que | Issue |
| --- | --- |
| O `clippy` do CI depende de um banco de pé | #125 |
| O OpenAPI precisa continuar gerando o mesmo cliente Dart | #126 |
| O contrato do erro de validação não está travado | #127 |
| Faltam testes de confirmação de e-mail, limites e validação | #128 |
| A entrega do e-mail de verificação não tem limite nem garantia | #132 |
| O reenvio da verificação não é atômico | #133 |
| A API não manda os headers de limite de requisições | #134 |
| O idioma do schema e o lugar do limite não estão registrados como decisão | #135 |
| Limpeza do código e do suporte de teste do cadastro | #137 |

# Riscos e dívida técnica

O que se sabe que está frágil ou em aberto, a consequência e a issue que acompanha. O estado de cada
issue mora no GitHub, nunca aqui.

## Riscos de arquitetura

| Risco | Consequência | Acompanha |
| --- | --- | --- |
| Não há hospedagem, Dockerfile nem deploy | o primeiro deploy é também o primeiro teste do Coolify, do Dockerfile e da migração no boot | [deployment.md](deployment.md) |
| Backup e recuperação sem decisão | a perda aceitável de dado e o tempo de volta não têm número | [OQ-11](../../product/requirements/open-questions.md#oq-11--backup-e-recuperação) |
| O offline ainda não tem spec | versão, UUID do cliente e idempotência precisam nascer com a primeira rota de escrita do produto, ou viram migração | [ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| Web, Android e iOS replicam o contrato da API à mão | uma mudança na API que um cliente não acompanha só aparece em runtime | [ADR 0019](../../decisions/0019-revisao-da-stack-dos-quatro-apps.md) |
| `main` e `staging` protegidas só por convenção | um push direto não é barrado pelo GitHub | [`AGENTS.md`](https://github.com/TheoOdawara/clinicore/blob/staging/AGENTS.md) → Branches |
| Parecer jurídico e custo da assinatura digital em aberto | o alcance da assinatura avançada pode mudar o fluxo do prontuário | [OQ-04](../../product/requirements/open-questions.md#oq-04--assinatura-digital-parecer-jurídico), [OQ-05](../../product/requirements/open-questions.md#oq-05--assinatura-digital-custos-e-requisitos) |
| Prazo de guarda do prontuário e direitos do titular sem decisão | o modelo de dados clínico nasce sem saber quando e como apaga | [OQ-09](../../product/requirements/open-questions.md#oq-09--prazo-de-guarda-do-prontuário), [OQ-10](../../product/requirements/open-questions.md#oq-10--direitos-do-titular-na-lgpd) |

## O que a autenticação precisa tratar

A primeira implementação da API, apagada em 2026-11-18, deixou estes problemas em aberto. A próxima nasce
tratando cada um, pela spec de autenticação.

| Problema | Consequência |
| --- | --- |
| Cadastro repetido de um e-mail não verificado | um terceiro cadastra o e-mail de outra pessoa antes dela e fica com a conta |
| Hash Argon2 sem teto de concorrência | uma rajada de logins esgota a CPU da API |
| IP do cliente ilegível no limite de requisições | todas as requisições caem no mesmo balde, e uma pessoa bloqueia todas |
| A cota de e-mail por endereço conta o pedido, não o envio | um e-mail que falhou consome a cota da pessoa |
| Login com a senha antiga durante um reset | a sessão aberta nesse intervalo sobrevive à troca de senha |
| Vínculo automático com o Google em endereço de que ele não é autoridade | uma conta Google de outro domínio assume a conta local |

Não há dívida técnica nem de segurança aberta: não há código.

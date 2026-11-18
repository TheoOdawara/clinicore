# Visão de runtime

Os cenários que a arquitetura precisa acertar, de ponta a ponta. Cada passo nomeia o bloco que age e com
quem ele fala.

| Cenário | O que mostra | Estado |
| --- | --- | --- |
| [Cadastro e confirmação de e-mail](sign-up-and-email-verification.md) | uma escrita com e-mail, sem revelar se o endereço já existe | a API existe; as telas no M1 |
| [Ciclo de vida da sessão](session-lifecycle.md) | login, refresh com rotação, logout e revogação imediata | a API existe; as telas no M1 |

Os cenários de escrita offline e de recusa de conflito entram aqui com a spec da agenda, no M1. O
conceito está em [`../concepts/offline.md`](../concepts/offline.md).

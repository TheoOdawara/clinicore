# User

Uma pessoa que entra no sistema. Marco: existe.

| Atributo | Tipo | Obrigatório | Domínio · regra |
| --- | --- | --- | --- |
| `id` | uuid | sim | gerado pelo banco |
| `name` | texto | sim | o nome que a pessoa informou |
| `email` | texto | sim | único, sempre em minúsculas |
| `email_verified` | booleano | sim | nasce `false`; vira `true` na confirmação |
| `image` | texto | não | URL da foto |
| `created_at` | data e hora | sim | — |
| `updated_at` | data e hora | sim | — |

- **Identidade:** `id`. O e-mail é único, mas pode mudar; a chave não.
- **Relações:** Account (1,N) — apagar o usuário apaga as contas. Session (0,N) — apagar o usuário
  apaga as sessões, e por isso elas são revogadas antes do `DELETE`.
- **Invariantes:** o e-mail está sempre em minúsculas; não existem dois usuários com o mesmo e-mail;
  só um usuário com e-mail verificado abre sessão.

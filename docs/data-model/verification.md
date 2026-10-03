# Verification

Um token de uso único mandado a um endereço de e-mail. Marco: a confirmação de e-mail existe; a
redefinição de senha entra no M1, com a #119.

| Atributo | Tipo | Obrigatório | Domínio · regra |
| --- | --- | --- | --- |
| `id` | uuid | sim | gerado pelo banco |
| `email` | texto | sim | o endereço que recebeu o link |
| `purpose` | enumeração | sim | `email_verification` ou `password_reset` |
| `token_hash` | texto | sim | único; hash do segredo que vai no link |
| `expires_at` | data e hora | sim | 1 hora depois da emissão, na confirmação de e-mail |
| `consumed_at` | data e hora | não | preenchido no uso |
| `created_at` | data e hora | sim | — |

- **Identidade:** `id`. A busca é sempre pelo `token_hash`, que também é único.
- **Relações:** User (0,1), pelo endereço de e-mail e sem chave estrangeira — apagar o usuário não apaga
  a verificação, que some pela purga quando vence.
- **Invariantes:** o segredo nunca é guardado, só o hash; token vencido ou já consumido não confirma
  nada; confirmar um token consome todas as verificações pendentes do mesmo endereço.

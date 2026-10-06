# Account

Um jeito de um usuário provar quem é: a senha, ou a conta Google dele. Marco: existe.

| Atributo | Tipo | Obrigatório | Domínio · regra |
| --- | --- | --- | --- |
| `id` | uuid | sim | gerado pelo banco |
| `user_id` | uuid | sim | o usuário dono |
| `provider` | enumeração | sim | `credential` ou `google` |
| `provider_account_id` | texto | não | o identificador do usuário no provedor; vazio na credencial |
| `password_hash` | texto | só na credencial | hash Argon2 da senha |
| `created_at` | data e hora | sim | — |
| `updated_at` | data e hora | sim | — |

- **Identidade:** `id`. O par usuário e provedor também é único.
- **Relações:** User (1,1) — a conta não existe sem o usuário e some com ele.
- **Invariantes:** um usuário tem no máximo uma conta por provedor; uma conta de provedor externo
  pertence a um usuário só; toda conta de credencial tem hash de senha.

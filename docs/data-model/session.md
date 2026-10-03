# Session

Um login ativo de um usuário num navegador ou num aparelho. Marco: existe.

| Atributo | Tipo | Obrigatório | Domínio · regra |
| --- | --- | --- | --- |
| `id` | uuid | sim | gerado pelo banco; vai dentro do access token |
| `user_id` | uuid | sim | o usuário dono |
| `refresh_token_hash` | texto | sim | hash do refresh token atual; troca a cada rotação |
| `expires_at` | data e hora | sim | renovado a cada rotação: 24 horas no web, 7 dias no app |
| `ip_address` | endereço IP | não | de onde o login veio |
| `user_agent` | texto | não | cortado em 512 caracteres |
| `client` | enumeração | sim | `web` ou `mobile`; decide o transporte do token |
| `created_at` | data e hora | sim | conta o teto de 30 dias |
| `updated_at` | data e hora | sim | — |

- **Identidade:** `id`.
- **Relações:** User (1,1) — a sessão some com o usuário.
- **Invariantes:** um usuário tem no máximo 5 sessões, e abrir a sexta apaga a mais antiga; nenhuma
  sessão vive mais de 30 dias; o refresh token em si nunca é guardado, só o hash; toda sessão apagada
  antes de vencer é escrita antes na denylist do Redis
  ([ADR 0012](../decisions/0012-revogacao-de-sessao-antes-de-apagar.md)).

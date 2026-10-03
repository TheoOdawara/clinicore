# EmailDispatch

O registro de um e-mail enviado a um endereço. Existe só para limitar o envio. Marco: existe.

| Atributo | Tipo | Obrigatório | Domínio · regra |
| --- | --- | --- | --- |
| `id` | uuid | sim | gerado pelo banco |
| `email` | texto | sim | o endereço que recebeu |
| `kind` | enumeração | sim | `email_verification` ou `password_reset` |
| `created_at` | data e hora | sim | o momento do envio |

- **Identidade:** `id`.
- **Relações:** User (0,1), pelo endereço de e-mail e sem chave estrangeira — o limite vale para o
  endereço, exista ou não uma conta nele.
- **Invariantes:** um endereço recebe no máximo 5 e-mails do mesmo tipo em 24 horas, e no máximo 1 a
  cada 60 segundos; a contagem e o registro acontecem sob o mesmo lock por endereço, então dois pedidos
  simultâneos não passam os dois.

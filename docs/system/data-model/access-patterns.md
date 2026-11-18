# Padrões de acesso

Cada operação que toca o banco, o que ela lê e escreve e por onde filtra. É daqui que saem os índices de
[storage.md](storage.md).

O volume esperado é o de duas clínicas piloto: dezenas de usuários e poucas sessões por usuário. Nenhuma
operação abaixo percorre uma tabela inteira, fora a purga.

| Operação | Lê | Escreve | Filtro e ordem |
| --- | --- | --- | --- |
| Cadastro | — | `users`, `accounts`, `email_dispatches`, `verifications`, `email_outbox`, numa transação | conflito por `users.email` |
| Reservar um envio | `email_dispatches` | `email_dispatches` | por `email` e `kind`, dentro da janela de 24 horas, sob lock por endereço |
| Pedir novo link | `users` | `email_dispatches`, `verifications`, `email_outbox`, numa transação | por `email`, depois de reservar o envio |
| Confirmar e-mail | `verifications` | `verifications`, `users` | por `token_hash`; depois consome todas as pendentes do `email`, sob lock por endereço |
| Pedir redefinição de senha | `users` | `email_dispatches`, `verifications`, `email_outbox`, numa transação | por `email`, depois de reservar o envio |
| Redefinir a senha | `verifications`, `users` (com lock de linha) | `accounts`, `verifications`, `sessions` | por `token_hash`; grava ou cria a conta de senha, consome as redefinições pendentes do `email` e apaga as sessões por `user_id` |
| Trocar a senha | `accounts`, `users` (com lock de linha) | `accounts`, `sessions` | por `user_id` e `provider = credential`; apaga as sessões do usuário menos a do pedido |
| Login | `users`, `accounts` | — | por `email` e `provider = credential` |
| Abrir sessão | `users` (com lock de linha) | `sessions` | insere, e apaga as sessões do usuário além das 5 mais recentes, por `user_id` em ordem de `created_at` |
| Ler o usuário da sessão | `users` | — | por `id` |
| Refresh | `sessions` (com lock de linha) | `sessions` | por `id` |
| Logout | — | `sessions` | por `id` |
| Purga diária, às 03:00, no worker | — | `sessions`, `verifications`, `email_dispatches`, `email_outbox` | por `expires_at` e por `created_at` |
| Reservar e-mail, no worker | `email_outbox` | `email_outbox` | até 10 linhas `pending` com `next_attempt_at` e `expires_at` em ordem, `FOR UPDATE SKIP LOCKED` |
| Resultado do envio, no worker | — | `email_outbox` | por `id`: apaga a enviada, adia ou marca `failed` a que falhou |

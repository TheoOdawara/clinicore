# Cadastro e confirmação de e-mail

Uma pessoa cria a conta e confirma o endereço. Cobre `FR-ACC-07`. A resposta nunca revela se o e-mail
já tinha conta.

## Cadastro

1. **O cliente** (web ou app) manda `POST /users` para a **API**, com nome, e-mail e senha.
2. **A API** passa a requisição pelos guards, nesta ordem: cliente, origem, limite por IP no **Redis** e
   validação do corpo.
3. **A API** gera o hash Argon2 da senha e um segredo aleatório para o link.
4. **A API** grava no **PostgreSQL**, numa transação, o usuário, a conta de credencial, a verificação
   com o hash do segredo e o registro do envio. Um e-mail que já existe não cria nada, por
   `ON CONFLICT … DO NOTHING`.
5. **A API**, só quando a conta foi criada, entrega ao **servidor SMTP** o e-mail com o link
   `<APP_ORIGIN>/verify-email?token=<segredo>`.
6. **A API** responde `202`, sem corpo, nos dois casos.

## Confirmação

1. **A pessoa** abre o link, e **o cliente** manda `POST /email-verifications/confirmation` para a
   **API**, com o token no corpo.
2. **A API** conta a requisição por IP e pela faixa de rede no **Redis**, porque o corpo só traz o token.
3. **A API** consome a verificação no **PostgreSQL** pelo hash do token e marca o e-mail como
   verificado.
4. **A API** responde `204`. Token desconhecido responde `400 invalid-token`, e token vencido,
   `400 token-expired`.

## Reenvio

1. **O cliente** manda `POST /email-verifications` para a **API**, com o e-mail.
2. **A API** reserva um envio para o endereço no **PostgreSQL**, serializado por
   `pg_advisory_xact_lock`. Sem envio disponível na janela de 24 horas, nada sai.
3. **A API**, se o endereço é de uma conta ainda não verificada, grava uma verificação nova e entrega o
   e-mail ao **servidor SMTP**.
4. **A API** responde `202`, sem corpo, em todos os casos.

Um login com senha correta numa conta não verificada dispara o mesmo reenvio e responde
`403 email-not-verified`.

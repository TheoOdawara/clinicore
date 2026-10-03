# Ciclo de vida da sessão

Login, uso, refresh e logout. Cobre `FR-ACC-08` e `FR-ACC-12`. A sessão é uma linha da tabela
`sessions`; o cliente carrega um access token de 15 minutos e um refresh token rotacionado, que vive 24
horas no web e 7 dias no app. Nenhuma sessão passa de 30 dias, com ou sem refresh.

O transporte depende do header `Clinicore-Client`: ausente é o web, com os tokens em cookie; `mobile` é
o app, com os tokens no corpo e o access token no `Authorization`.

## Login

1. **O cliente** manda `POST /sessions` para a **API**, com e-mail e senha.
2. **A API** conta a requisição por IP e as falhas de senha por e-mail no **Redis**: 10 falhas em 15
   minutos bloqueiam o endereço.
3. **A API** busca a credencial no **PostgreSQL** e confere a senha. Sem conta, confere contra um hash
   que nunca casa, para o tempo de resposta não revelar se o e-mail existe.
4. **A API** abre a sessão no **PostgreSQL** com o hash do refresh token. Ficam as 5 sessões mais
   recentes do usuário; as mais antigas são apagadas e revogadas no **Redis**, na mesma transação.
5. **A API** responde `201` com o usuário, e com os tokens em cookie para o web ou no corpo para o app.
   Senha errada responde `401 invalid-credentials`.

## Requisição autenticada

1. **O cliente** chama uma rota com o access token, no cookie ou no `Authorization`.
2. **A API** confere a assinatura e a validade do token, sem ir ao PostgreSQL, e consulta a denylist no
   **Redis**.
3. **A API** conta a requisição por sessão no **Redis** e segue para o handler.

## Refresh

1. **O cliente** manda `POST /sessions/current/tokens` para a **API**, com o refresh token.
2. **A API** conta a requisição pela sessão e consulta a denylist no **Redis**.
3. **A API** troca o hash do refresh token no **PostgreSQL**, só se o hash apresentado é o atual.
4. **A API** responde `204` com cookies novos para o web, ou `200` com os tokens no corpo para o app.
5. Um refresh token antigo que volta é reuso: **a API** revoga a sessão no **Redis**, apaga-a no
   **PostgreSQL** e responde `401 session-reused`.

## Logout e revogação

1. **O cliente** manda `DELETE /sessions/current` para a **API**.
2. **A API** escreve `auth:revoked:<sessão>` no **Redis**, com vida de 7 dias.
3. **A API** apaga a sessão no **PostgreSQL** e responde `204`.

A ordem é a da [ADR 0012](../../decisions/0012-revogacao-de-sessao-antes-de-apagar.md): com o Redis
fora, nada é apagado e a rota responde `503`.

## Purga

De hora em hora, **a API** apaga no **PostgreSQL** a sessão vencida ou passada do teto de 30 dias, a
verificação vencida e o registro de envio de mais de 24 horas. Sobe para o worker com a #71.

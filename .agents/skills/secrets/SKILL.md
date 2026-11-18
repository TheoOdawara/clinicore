---
name: secrets
description: Como as variáveis de ambiente de desenvolvimento chegam ao Clinicore, pelo Infisical, sem nunca imprimir um valor. Use ao rodar algo que precisa de variável, ao criar uma variável nova, ou quando uma porta de dev conflitar.
---

# secrets

- **Os valores de desenvolvimento moram no Infisical**, no ambiente `dev` do projeto `Clinicore` da org
  `ATNexusLab`, na pasta do app (`/api`). Ninguém manda variável para ninguém.
- **Todo comando que precisa de variável roda dentro do `infisical run --path=/api --`**: `cargo run`,
  `docker compose up`, `sqlx migrate run`, `cargo sqlx prepare` e os testes. O `--env` padrão já é `dev`.
- **Só roda dentro do `infisical run` o que precisa.** E nunca `infisical secrets`, `infisical export`,
  `env`, `printenv` nem `docker compose config`: todos imprimem os valores.
- **O `.infisical.yaml` é commitado**: liga a pasta ao projeto e não tem segredo.
- **O `.env` de cada app é mantido no disco, fora do git, e sempre igual ao Infisical.** Toda mudança
  de valor vai para os dois ao mesmo tempo; um sem o outro é um ambiente quebrado. O binário não lê o
  `.env`; o `docker compose` e o `sqlx-cli` o leem sozinhos quando rodam sem o `infisical run`.
- **Valor só da sua máquina é override pessoal**, com
  `infisical secrets set <NOME>=<valor> --type=personal`: o caso de uma porta já ocupada.
- **Variável nova entra no `.env.example`, no `.env` local, no leitor único do app e no Infisical ao mesmo
  tempo.** O `.env.example` tem o nome e o formato, nunca o valor.
- **O repo é público.** Valor real nunca entra em arquivo commitado, nem no CI de PR, que só recebe valor
  descartável de teste.

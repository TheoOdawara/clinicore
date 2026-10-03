# Configuração e segredos

A mesma regra vale nos quatro apps: toda configuração é obrigatória, lida num lugar só e validada antes
de o app fazer qualquer coisa. Nenhuma tem valor padrão.

| App | De onde vem | Onde é lida | Se falta |
| --- | --- | --- | --- |
| `apps/api` | variáveis do processo, injetadas pelo `infisical run` em dev e pelo orquestrador fora dele | `crates/core/src/config.rs` | escreve `Invalid environment:` com uma linha por variável e sai com código 1, antes do bind |
| `apps/web` | variáveis do build do Vite | `src/shared/env/` | a validação do schema falha |
| `apps/mobile` | `--dart-define-from-file=config/<ambiente>.json` | `lib/shared/env/env.dart` | o app sobe numa tela que só mostra os nomes que faltam |
| `apps/site` | não lê configuração | — | — |

## Segredos

- **Os valores de desenvolvimento moram no Infisical**, no projeto `Clinicore`, ambiente `dev`, pasta
  `/api`. O `.infisical.json` commitado liga a pasta ao projeto e não tem segredo.
- **O repo só lista nome e formato**: `apps/api/.env.example`, `apps/web/.env.example` e
  `apps/mobile/config/example.json`.
- **O site não recebe segredo**, e o web e o app só recebem valor que pode ser público, porque o que
  entra no build chega ao aparelho de quem usa.

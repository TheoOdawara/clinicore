# Configuração e segredos

A mesma regra vale nos cinco apps: toda configuração é obrigatória, lida num lugar só e validada antes
de o app fazer qualquer coisa. Nenhuma tem valor padrão.

| App | De onde vem | Onde é lida | Se falta |
| --- | --- | --- | --- |
| `apps/api` | variáveis do processo, injetadas pelo `infisical run` em dev e pelo orquestrador fora dele | `crates/core/src/config.rs` | escreve `Invalid environment:` com uma linha por variável e sai com código 1, antes do bind |
| worker | as mesmas, pela mesma pasta `/api` do Infisical, mas só as oito que ele usa: `APP_ENV`, `LOG_LEVEL`, `DATABASE_URL` e as cinco do SMTP, que a API não lê | `crates/core/src/config.rs`, no `WorkerConfig` | o mesmo da API, antes de abrir conexão |
| `apps/web` | valores do build, pelo `--define` do Angular | `src/app/core/config/app-config.ts` | a validação do Zod falha no boot |
| `apps/android` | `config/<ambiente>.properties`, validado pelo Gradle e posto no `BuildConfig` | `app/.../AppConfig.kt` | o build falha com a chave vazia |
| `apps/ios` | `Config/<configuração>.xcconfig`, pelo `Info.plist` | `Clinicore/AppConfig.swift` | o app não sobe e diz a chave que falta |
| `apps/site` | não lê configuração | — | — |

## Segredos

- **Os valores de desenvolvimento moram no Infisical**, no projeto `Clinicore` da org `ATNexusLab`,
  ambiente `dev`, pasta `/api`. O `.infisical.yaml` commitado liga a pasta ao projeto e não tem segredo.
- **O `.env` local é mantido sempre igual ao Infisical**, fora do git: toda mudança vai para os dois.
- **O repo só lista nome e formato**: `apps/api/.env.example`, `apps/web/.env.example`,
  `apps/android/config/example.properties` e `apps/ios/Config/Example.xcconfig`.
- **O site não recebe segredo**, e o web e os apps nativos só recebem valor que pode ser público,
  porque o que entra no build chega ao aparelho de quem usa.

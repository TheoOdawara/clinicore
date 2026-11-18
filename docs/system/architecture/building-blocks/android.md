# `apps/android`

O sistema da clínica como app nativo de Android, em Kotlin e Jetpack Compose, com as mesmas telas do web
e funcionando sem conexão ([ADR 0022](../../../decisions/0022-apps-nativos-em-kotlin-e-swift.md),
[ADR 0007](../../../decisions/0007-app-nativo-em-flutter-com-offline.md) nos itens 3 a 9). Fala com a API
mandando `Clinicore-Client: mobile` e o token no `Authorization`.

**Nunca faz:** gerar código do OpenAPI da API, deixar uma tela falar com a rede ou com o banco sem passar
pelo repository, ler configuração fora do `AppConfig.kt`.

Existem o shell, o tema sobre os tokens, a fonte, as fronteiras de módulo conferidas pelo Gradle e pelo
Konsist, e o CI. Os demais blocos nascem com a #9.

## Blocos internos

| Bloco | Onde | Responsabilidade | Estado |
| --- | --- | --- | --- |
| App | `app/` | a Activity, a navegação raiz e as abas | existe a Activity com o shell: a barra com o nome e o conteúdo vazio |
| Domínio | `core/model/` | os modelos, em Kotlin puro | M1 |
| Rede | `core/network/` | Retrofit e OkHttp, o refresh único no 401 e o Problem Details | M1 |
| Banco local | `core/database/` | Room com SQLCipher: o dado clínico e, num arquivo à parte, a fila de escrita | M1 |
| Cofre | `core/security/` e `core/datastore/` | as chaves no Keystore, o token cifrado e os apagamentos | M1 |
| Dados | `core/data/` | os repositories, a única porta dos dados para as telas | M1 |
| Sincronização | `core/sync/` | o WorkManager que esvazia a fila em ordem, e as Pendências | M1 |
| UI base | `core/designsystem/` e `core/ui/` | o tema Material 3 sobre os tokens e os componentes | existe o tema em `core/designsystem/`, claro e escuro, com a Atkinson Hyperlegible Next; os componentes e o `core/ui/` são M1 |
| Features | `feature/<feature>/` | as chaves de navegação (`api`) e as telas com o ViewModel (`impl`) | M1 |

## Para mudar com segurança

As dependências entre módulos, conferidas pelo Gradle e pelo Konsist, e os nomes estão no
[`apps/android/AGENTS.md`](https://github.com/TheoOdawara/clinicore/blob/staging/apps/android/AGENTS.md). O conceito de offline está em
[`../concepts/offline.md`](../concepts/offline.md).

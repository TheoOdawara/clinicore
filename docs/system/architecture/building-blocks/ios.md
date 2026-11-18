# `apps/ios`

O sistema da clínica como app nativo de iOS, em Swift e SwiftUI, com as mesmas telas do web e
funcionando sem conexão ([ADR 0022](../../../decisions/0022-apps-nativos-em-kotlin-e-swift.md),
[ADR 0007](../../../decisions/0007-app-nativo-em-flutter-com-offline.md) nos itens 3 a 9). Fala com a API
mandando `Clinicore-Client: mobile` e o token no `Authorization`.

**Nunca faz:** gerar código do OpenAPI da API, deixar uma tela tocar o SwiftData, o Keychain ou o
`URLSession`, guardar dado offline num aparelho sem código de desbloqueio.

O app ainda não existe; nasce no M1.

## Blocos internos

| Bloco | Onde | Responsabilidade | Estado |
| --- | --- | --- | --- |
| App | `Clinicore/` | o `@main`, o `AppContainer`, o `Router` e as abas | M1 |
| Domínio | `Packages/Sources/Models/` | os modelos, `Sendable` | M1 |
| Rede | `Packages/Sources/Networking/` | o `APIClient`, o refresh único no 401 e o Problem Details | M1 |
| Banco local | `Packages/Sources/Persistence/` | SwiftData sob Data Protection: o dado clínico e, num arquivo à parte, a fila | M1 |
| Cofre | `Packages/Sources/Security/` | o token no Keychain, a checagem do código do aparelho e os apagamentos | M1 |
| Dados | `Packages/Sources/Repositories/` | a única porta dos dados para as telas | M1 |
| Sincronização | `Packages/Sources/Sync/` | o `SyncEngine` que esvazia a fila em ordem, e as Pendências | M1 |
| UI base | `Packages/Sources/DesignSystem/` | o tema sobre os tokens e os componentes | M1 |
| Features | `Packages/Sources/<Feature>Feature/` | as telas, cada uma com `State`, `ViewModel` e `View` | M1 |

## Para mudar com segurança

As dependências entre targets, conferidas pelo `Package.swift` e pelo SwiftLint, e os nomes estão no
[`apps/ios/AGENTS.md`](https://github.com/TheoOdawara/clinicore/blob/staging/apps/ios/AGENTS.md). O conceito de offline está em
[`../concepts/offline.md`](../concepts/offline.md).

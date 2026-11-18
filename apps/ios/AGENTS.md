# `apps/ios` — forma e regras

Aditivo ao `AGENTS.md` da raiz. Stack na ADR 0022, forma na 0026, offline na 0007 (itens 3 a 9, e o 7
emendado pela 0022). App nativo em Swift e SwiftUI, com as mesmas telas do web e offline completo.

**O app ainda não existe**: nasce no scaffold, nesta forma.

## Estrutura

```
apps/ios/
├── Clinicore.xcodeproj           só o target do app
├── Clinicore/                    o app, em pasta sincronizada
│   ├── ClinicoreApp.swift        @main: monta o AppContainer, registra as BGTasks
│   ├── AppContainer.swift        cria e injeta tudo
│   ├── AppConfig.swift           único leitor da configuração
│   ├── Navigation/               Router.swift, Route.swift, RootView.swift (TabView, NavigationStack por aba)
│   ├── Resources/                Assets.xcassets, Localizable.xcstrings, PrivacyInfo.xcprivacy
│   └── Info.plist  Clinicore.entitlements
├── Config/                       Example.xcconfig commitado; Debug, Staging e Release fora do git
├── ClinicoreUITests/<feature>/   XCUITest das telas
├── ClinicoreE2ETests/            XCUITest contra a API local
└── Packages/
    ├── Package.swift             um pacote, vários targets
    ├── Sources/
    │   ├── Models/               structs do domínio, Sendable
    │   ├── Networking/           APIClient, o actor de refresh, APIError, *Request, *Response
    │   ├── Persistence/          SwiftData: @Model internos, @ModelActor, clinical.store, outbox.store
    │   ├── Security/             TokenStore (Keychain), código do aparelho (LAContext), apagamentos
    │   ├── Repositories/         a única porta dos dados para as telas
    │   ├── Sync/                 SyncEngine, NWPathMonitor, Pendências
    │   ├── DesignSystem/         tema com os tokens, componentes base, ícones
    │   ├── TestSupport/          fakes, fixtures de JSON, stub de URLProtocol
    │   └── <Feature>Feature/<Tela>/   <Tela>State.swift, <Tela>ViewModel.swift, <Tela>View.swift
    └── Tests/<Target>Tests/      Swift Testing
```

## Fronteiras

As dependências do `Package.swift` impõem a fronteira: um target só importa o que declarou.

| Target | Depende de |
| --- | --- |
| `Models` | nada |
| `Networking`, `Security` | `Models` |
| `Persistence` | `Models`, `Security` |
| `Repositories` | `Networking`, `Persistence`, `Models` |
| `Sync` | `Repositories`, `Networking`, `Persistence` |
| `<X>Feature` | `Repositories`, `DesignSystem`, `Models`; nunca outra feature |
| o app | tudo; liga as features pelo `Router` |

Regras próprias do SwiftLint conferem o resto: `import SwiftData` só em `Persistence`, `URLSession` só em
`Networking`, `SecItem` e `LAContext` só em `Security`, e `ViewModel` só em `*Feature`, sempre com
`@MainActor @Observable`.

## Padrões

- **A View guarda o ViewModel em `@State private var viewModel`.** O ViewModel é
  `@MainActor @Observable final class`, com `private(set) var state` e uma função por evento, com nome
  de verbo; o `State` é `struct`.
- **A feature não conhece o `Route`**: recebe closures (`onSelectPatient: (Patient.ID) -> Void`), e o
  app decide o destino.
- **Injeção manual pelo `init`**, no `AppContainer`; `@Environment` só para o `Router` e a sessão.
- **O banco local é a fonte da verdade**, e toda escrita vai para a fila. Nada sincroniza com o aparelho
  bloqueado, e sem código de desbloqueio nada fica offline.
- **Só é `public` o que outro target usa**; o resto é `internal`. Modelo do domínio é `struct` imutável.
- **Swift 6 com concorrência estrita**, e aviso é erro.

## Nomes

Arquivo `.swift` em PascalCase com o nome do tipo principal (regra do Swift, cobrada pelo `file_name` do
SwiftLint); extensão que adota protocolo em `Tipo+Protocolo.swift`. Target de feature em
`<Feature>Feature`.

## Caminhos críticos

100% das linhas novas (ADR 0023): `Security`, `Networking`, `Sync` e o `outbox.store` em `Persistence`.

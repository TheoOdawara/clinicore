# `apps/ios` — forma e regras

Aditivo ao `AGENTS.md` da raiz. Stack na ADR 0022, forma na 0026, offline na 0007 (itens 3 a 9, e o 7
emendado pela 0022). App nativo em Swift e SwiftUI, com as mesmas telas do web e offline completo.

## Estrutura

```
apps/ios/
├── Clinicore.xcodeproj           o target do app e o de UI test; o pacote entra como pasta
├── Clinicore.xctestplan          os testes do pacote e os XCUITest, com cobertura
├── .swiftlint.yml  .swift-format
├── ArchitectureFixtures/         uma fonte que viola cada regra própria do SwiftLint, fora de qualquer target
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

- **Toda regra própria do SwiftLint tem uma fonte em `ArchitectureFixtures/`**, num caminho que a viola, e
  o gate de fronteiras exige que cada regra dispare nela.
- **A acessibilidade de cada tela é conferida pelo `performAccessibilityAudit()`** no XCUITest, no claro,
  no escuro e no maior Dynamic Type; o contraste dos tokens, pelo `ContrastTests` do `DesignSystem`.
- **Texto que precisa crescer com o Dynamic Type fica fora da barra de navegação**: o iOS limita o
  tamanho dele lá, e o audit reprova.
- **Target de teste novo entra no `Clinicore.xctestplan`**, senão o `xcodebuild test` não o roda.
- **Até o `AppConfig` chegar (#10), o `Info.plist` é gerado pelo build** (`GENERATE_INFOPLIST_FILE`), e não
  há `Config/`, `Clinicore.entitlements` nem as configurações `Staging` e `Release`.
- `shortcut: o app não tem ícone porque a marca ainda não tem logo (docs/product/design/tokens.md), o
  AppIcon entra com ele`

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

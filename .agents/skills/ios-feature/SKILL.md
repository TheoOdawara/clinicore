---
name: ios-feature
description: O passo a passo de uma feature nova ou de uma tela nova no apps/ios do Clinicore, em Swift e SwiftUI. Use ao criar ou mudar feature, tela, ViewModel, repository ou modelo local no apps/ios.
---

# ios-feature

A forma (targets, fronteiras, nomes) está no `apps/ios/AGENTS.md`. Vale a partir da issue de scaffold do
iOS.

## Feature nova

1. Declarar o target `<Feature>Feature` no `Packages/Package.swift`, dependendo só de `Repositories`,
   `DesignSystem` e `Models`, e o target de teste `<Feature>FeatureTests`.
2. Adicionar o produto ao target do app no `Clinicore.xcodeproj`.
3. No app, os casos novos entram no `Route`, e o `RootView` liga cada caso à tela da feature, passando
   as closures de navegação.

## Dado novo

1. O modelo do domínio entra em `Models` (`Patient`), como `struct` `Sendable`.
2. A resposta da API entra em `Networking` (`PatientResponse`), `Codable`, com a fixture de JSON em
   `TestSupport`.
3. O `@Model` entra em `Persistence`, `internal`, com a migração no `VersionedSchema`; o `@ModelActor`
   expõe só funções que devolvem `Models`.
4. O repository entra em `Repositories`: o protocolo `PatientsRepository` e a implementação, que lê do
   banco local e escreve na fila.
5. O fake do repository entra em `TestSupport`.

## Tela nova

1. Uma pasta por tela no target da feature, com `<Tela>State.swift`, `<Tela>ViewModel.swift` e
   `<Tela>View.swift`.
2. O ViewModel é `@MainActor @Observable final class`, recebe o repository pelo `init` e tem
   `private(set) var state` e uma função por evento.
3. A View guarda o ViewModel em `@State private var viewModel` e recebe as closures de navegação.
4. A tela usa só os componentes do `DesignSystem`.

## Testes

- ViewModel e repository: Swift Testing em `Tests/<Target>Tests/`, com os fakes do `TestSupport` e o
  stub de `URLProtocol`.
- Tela e Keychain real: XCUITest em `ClinicoreUITests/<feature>/`.
- Fluxo: `ClinicoreE2ETests`, contra a API local.

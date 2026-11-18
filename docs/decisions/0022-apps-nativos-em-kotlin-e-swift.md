# 0022. Apps nativos em Kotlin e Swift

- Status: accepted, emendada pela 0026 na estrutura e nas convenções
- Date: 2026-11-17
- Substitui: 0007 na plataforma e na stack (a tabela e os itens 1 e 2) e 0019 na tabela do `apps/mobile`
- Emenda a: 0007 no item 7, só no iOS (sem código de desbloqueio no aparelho, nada fica offline)
- Mantém: os itens 3 a 6, 8 e 9 da 0007, que são o comportamento offline e valem para os dois apps

## Context

A 0007 criou o `apps/mobile` em Flutter, e o scaffold dele existe. Nenhuma tela foi escrita. O dono do
produto achou o projeto Flutter desorganizado, com cada coisa de cada plataforma num lugar diferente, e
decidiu por app nativo de verdade: Kotlin no Android e Swift no iOS.

Cada plataforma foi decidida pelas próprias regras, com a documentação do Google e da Apple, os apps em
produção e o mercado, e com cada versão conferida em 2026-11-17. A paridade entre os apps é de
comportamento e de telas, que é requisito de produto; a forma do código de um app não segue a do outro.

## Decision

**O `apps/mobile` dá lugar a dois apps independentes: `apps/android` e `apps/ios`.** Cada um tem o
próprio build, os próprios gates e o próprio `AGENTS.md`.

### `apps/android`

| Papel | Escolha | Versão em 2026-11-17 |
| --- | --- | --- |
| Build | Gradle com Kotlin DSL, version catalog e wrapper commitado · Android Gradle Plugin · JDK 17 | 9.9.1 · 9.5.1 |
| Linguagem | Kotlin | 2.4.21 |
| SDK | `compileSdk` e `targetSdk` 37 · `minSdk` 24 | — |
| Módulos | Multimódulo desde o início: `app`, `core/model`, `core/data`, `core/ui`, `feature/<feature>` e `build-logic` com plugins de convenção | — |
| UI | Jetpack Compose com Material 3 | BOM 2026.09.00 |
| Arquitetura | O guia oficial: ViewModel com `StateFlow<UiState>`, uma função pública por evento, repository e o banco local como fonte da verdade; camada de domínio só quando uma regra se repete | — |
| Navegação | Navigation 3 | 1.2.1 |
| Injeção | Hilt · `hilt-lifecycle-viewmodel-compose` | 2.60.1 · 1.4.0 |
| HTTP | Retrofit sobre OkHttp; um Interceptor põe `Clinicore-Client: mobile` e o `Authorization`, e um Authenticator faz um único refresh no 401 | 3.0.0 · 5.5.1 |
| JSON | kotlinx.serialization, com `ignoreUnknownKeys` no app e decodificação estrita sobre fixtures nos testes | 1.13.0 |
| Banco | Room 3 com SQLCipher pelo `SQLCipherDriver`, a chave no Android Keystore | 3.0.3 · 4.20.0 |
| Fila offline | `outbox.db` em SQLCipher com chave própria, separado do `clinical.db`, esvaziado por WorkManager com trabalho único, restrição de rede, ordem e espera crescente | 2.12.0 |
| Tokens | DataStore com a chave AES-GCM no Android Keystore, num `TokenStore` | 1.3.0 |
| Ambientes | Flavors `dev`, `staging` e `prod`; `config/<ambiente>.properties` fora do git, com o `example` commitado; o Gradle reprova chave vazia; `AppConfig.kt` é o único leitor | — |
| Formato | ktfmt no estilo `kotlinlang`, pelo Spotless | 0.64 · 8.11.0 |
| Lint | detekt com compose-rules · Android Lint com `warningsAsErrors` · Kotlin com `allWarningsAsErrors` | 2.0.0-alpha.6 · 0.6.8 |
| Arquitetura conferida | A fronteira entre módulos pelo Gradle e as camadas dentro do módulo por testes do Konsist | 0.18.1 |
| Testes | JUnit 4 na JVM: coroutines-test, Turbine, Truth, Robolectric com Compose UI test para as telas, MockWebServer, `work-testing`, `hilt-android-testing` e `room3-testing`; fakes escritos à mão e MockK só onde o fake não compensa | 4.13.2 · 1.11.0 · 1.2.1 · 1.4.5 · 4.18 · 1.12.1 |
| Teste no emulador | Só o que a JVM não roda (SQLCipher, Keystore, migração real, apagamentos), com AndroidX Test e Orchestrator no Gradle Managed Devices, nas APIs 24 e 37 | 1.7.0 · 1.6.1 |
| E2E | Maestro, com os fluxos em YAML em `apps/android/e2e/`, contra a API local | 2.12.1 |
| Cobertura | Kover agregado | 0.9.9 |
| Dependências | Dependency locking do Gradle e `gradle/verification-metadata.xml` com SHA-256 | — |
| CVE | osv-scanner sobre os `gradle.lockfile` | 2.7.1 |

O detekt fica na alpha por decisão do dono do produto, até a 2.0 sair estável. A versão estável, 1.23.8,
foi feita para o Kotlin 2.0.21.

### `apps/ios`

| Papel | Escolha | Versão em 2026-11-17 |
| --- | --- | --- |
| Ferramentas | Xcode · Swift no language mode 6, com checagem estrita de concorrência | 27.2 · 6.4.2 |
| Mínimo | iOS 26 | — |
| UI | SwiftUI; UIKit só por `UIViewRepresentable` quando faltar algo | — |
| Projeto | `Clinicore.xcodeproj` commitado, só com o target do app e pastas sincronizadas, e um pacote SwiftPM local em `Packages/` com os targets `Core/Model`, `Core/Data`, `Core/UI` e `Feature/<feature>` | — |
| Arquitetura | ViewModel `@MainActor @Observable` com `private(set) var state` e uma função por evento; repository; o banco local como fonte da verdade; cada tela tem `State`, `ViewModel` e `View` | — |
| Navegação | `NavigationStack(path:)` por aba num `TabView`; um `Router` `@Observable` guarda as pilhas de `Route`, um `enum` `Hashable`; a feature recebe o destino como closure; deep link por `onOpenURL` | — |
| Injeção | Manual: um `AppContainer` no target do app e injeção pelo `init`; `@Environment` só para o `Router` e a sessão | — |
| HTTP | `URLSession` com `async/await` e um `APIClient` fino no `Core/Data`; um `actor` faz um único refresh no 401; o Problem Details vira um `APIError` tipado | — |
| JSON | `Codable`, aceitando data RFC 3339 com e sem fração de segundo | — |
| Banco | SwiftData, com os `@Model` presos aos repositories do `Core/Data` e escrita por `@ModelActor` | — |
| Cifragem | Data Protection com o entitlement `default-data-protection` em `NSFileProtectionComplete`; `clinical.store` e `outbox.store` separados e fora do backup | — |
| Fila offline | Um `SyncEngine` (`actor`) esvazia a fila em ordem; gatilhos: o app abrir, a rede voltar (`NWPathMonitor`), salvar, `beginBackgroundTask` ao sair e `BGAppRefreshTask` periódico | — |
| Tokens | `TokenStore` próprio sobre o `SecItem`, com `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` | — |
| Ambientes | Configurações `Debug`, `Staging` e `Release`, cada uma com o seu bundle ID; `Config/Example.xcconfig` commitado e os outros fora do git; os valores chegam pelo `Info.plist`, e `AppConfig.swift` é o único leitor | — |
| Formato | swift-format da Apple, que vem com o Xcode | 6.4.4 |
| Lint | SwiftLint com `--strict`, pelo plugin de SPM oficial, com regras próprias de arquitetura · `SWIFT_TREAT_WARNINGS_AS_ERRORS` | 0.65.1 |
| Testes | Swift Testing para a lógica, por pacote · XCUITest para as telas e o Keychain real | — |
| E2E | XCUITest, numa suíte `ClinicoreE2ETests` contra a API local | — |
| Cobertura | Nativa do Xcode e do SwiftPM | — |
| CVE | Alertas do Dependabot sobre o `Package.resolved` commitado | — |

O SwiftData é provisório. Ele volta a ser decidido se a migração de schema doer, se a concorrência do
Swift 6 atrapalhar ou se o volume de dados crescer; a saída é o GRDB ou o SQLiteData, e a troca fica
restrita ao `Core/Data`.

## Consequences

- **Toda issue de tela ou de comportamento do web passa a ter duas gêmeas**, uma no `apps/android` e uma
  no `apps/ios`. A label `area:mobile` dá lugar a `area:android` e `area:ios`.
- **Sync, fila, cifragem e cada tela são escritos duas vezes além do web.** É o custo que a 0007
  recusou, e o dono do produto aceita.
- **No iOS, aparelho sem código de desbloqueio não guarda dado offline**: o app confere com o
  `LAContext`, funciona só online e avisa a pessoa. A Data Protection, e com ela o Keychain, não protege
  nada sem código, então a 0007 aceitava um risco que no iOS fica fechado. No Android, o SQLCipher com a
  chave no Keystore segue o item 7 da 0007.
- **No iOS, nada sincroniza com o aparelho bloqueado**, porque o arquivo `.complete` fica ilegível. A
  regra das 72 horas conta a partir da última sincronização que deu certo.
- **O simulador não aplica a Data Protection.** A garantia de que o banco fica ilegível com o aparelho
  bloqueado é conferida num iPhone de verdade antes de cada versão.
- **O Keychain sobrevive à desinstalação do app.** Na primeira abertura depois de instalar, o app apaga
  qualquer token que encontrar.
- **O CI do iOS roda na imagem `xcode-27` do GitHub, que ainda é preview**, porque a estável só vai até o
  Xcode 26.6, que não tem o Swift 6.4.
- **A skill de feature de cada app** (`android-feature` e `ios-feature`) carrega o passo a passo, e o
  `AGENTS.md` de cada um carrega a forma.

## Alternatives considered

- **Manter o Flutter (0007)** · rejeitada pelo dono do produto.
- **Kotlin Multiplatform** · rejeitada: o pedido foi um app nativo em cada plataforma.
- **No Android: Room 2.8, ktlint, JUnit 6 e Espresso direto** · rejeitados: o Room 3 é estável e é
  para onde o Google leva o Room; o ktlint 2.0 só existe como tag no GitHub e brigaria com o ktfmt; o
  Robolectric e o Compose UI test rodam sobre o JUnit 4; o Compose UI test e o Maestro cobrem o que o
  Espresso faria.
- **No Android: módulo único** · rejeitado pelo dono do produto, em favor do multimódulo desde o início.
- **No iOS: TCA** · rejeitado: pesado, de terceiro e pouco adotado fora da própria Point-Free.
- **No iOS: XcodeGen ou Tuist** · rejeitados: o app tem um target só.
- **No iOS: Factory, swift-dependencies ou Swinject** · rejeitados: a injeção manual é checada pelo
  compilador.
- **No iOS: Alamofire** · rejeitado: o `URLSession` com `async/await` faz o mesmo sem dependência.
- **No iOS: GRDB com SQLCipher** · rejeitado por enquanto: o SQLCipher no iOS exige fork do GRDB e
  depende do mesmo código de desbloqueio que a Data Protection.
- **No iOS: Maestro** · rejeitado: no iOS ele roda sobre o próprio XCUITest.
- **No iOS: ViewInspector** · rejeitado: lê a estrutura interna do SwiftUI, que muda a cada versão.

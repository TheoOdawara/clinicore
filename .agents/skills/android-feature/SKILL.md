---
name: android-feature
description: O passo a passo de uma feature nova ou de uma tela nova no apps/android do Clinicore, em Kotlin e Compose. Use ao criar ou mudar feature, tela, ViewModel, repository ou tabela local no apps/android.
---

# android-feature

A forma (módulos, fronteiras, nomes) está no `apps/android/AGENTS.md`. O scaffold criou só o `app`, o
`core:designsystem` e os plugins `clinicore.android.application`, `.library` e `.compose`; o primeiro
módulo que precisar de `clinicore.jvm.library`, `clinicore.android.feature`, Room ou Hilt cria o plugin.

## Feature nova

1. Criar `feature/<feature>/api` com o plugin `clinicore.jvm.library` e as chaves de navegação
   (`<Tela>NavKey`), e `feature/<feature>/impl` com o plugin `clinicore.android.feature`.
2. Incluir os dois no `settings.gradle.kts` e o `impl` no `app`, e regravar os lockfiles e o
   `verification-metadata.xml` (skill `gates`). O módulo entra sozinho no relatório do Kover da raiz,
   porque a convenção o registra.
3. Em `impl/navigation/`, o `<Feature>EntryProvider.kt` liga cada chave à tela.

## Dado novo

1. O modelo do domínio entra em `core:model` (`Patient`), como `data class` imutável.
2. A resposta da API entra em `core:network` (`PatientResponse`), com `@Serializable`, e a fixture de
   JSON em `core:testing`.
3. A tabela entra em `core:database` (`PatientEntity`, `PatientDao`), com a migração do Room e o schema
   exportado em `schemas/`.
4. O repository entra em `core:data`: a interface `PatientsRepository` e a implementação `internal`
   `OfflineFirstPatientsRepository`, que lê do banco e escreve na fila do `outbox.db`.
5. O fake do repository entra em `core:testing`.

## Tela nova

1. Uma pasta por tela em `impl/`, com `<Tela>Screen.kt`, `<Tela>ViewModel.kt` e `<Tela>UiState.kt`.
2. O ViewModel é `@HiltViewModel`, recebe o repository e expõe `uiState: StateFlow<<Tela>UiState>` e uma
   função por evento.
3. O `<Tela>Route` coleta o estado com `collectAsStateWithLifecycle` e passa ao `<Tela>Screen`, que é
   sem estado.
4. A tela usa só os componentes do `core:designsystem` e do `core:ui`.

## Testes

- ViewModel: JUnit 4 com o fake do repository, coroutines-test e Turbine.
- Repository: MockWebServer e o `room3-testing`.
- Tela: Compose UI test sobre o Robolectric, para o comportamento e a semântica.
- Acessibilidade da tela: `androidTest` com `enableAccessibilityChecks()`, no emulador; o ATF não avalia
  Compose no Robolectric.
- Fluxo: Maestro em `e2e/<feature>/`.

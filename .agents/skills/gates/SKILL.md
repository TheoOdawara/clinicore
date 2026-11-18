---
name: gates
description: Os gates de cada app do Clinicore, com o comando exato e a ordem. Use antes de dar qualquer mudança por pronta, e para saber o que o CI vai rodar.
---

# gates

Cada comando roda do diretório indicado, do mais barato ao mais caro. Um gate que falha para a entrega:
o resultado é relatado com a saída.

## Documentação

Da raiz do repo:

| Gate | Comando |
| --- | --- |
| Build do site | `uvx zensical==0.1.2 build --clean` |

## `apps/api`

De `apps/api/`, com o compose de pé (`infisical run --path=/api -- docker compose up -d --wait`). O CI
roda os mesmos, no `ci-api.yml`.

| Gate | Comando |
| --- | --- |
| Formato | `cargo fmt --all --check` |
| Dependências | `cargo deny check` |
| Lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Consultas offline | `infisical run --path=/api -- cargo sqlx prepare --workspace --check` |
| Testes e cobertura | `infisical run --path=/api -- cargo llvm-cov nextest --workspace --locked --lcov --output-path target/lcov.info --ignore-filename-regex 'main\.rs'` |
| Linhas novas | da raiz: `uvx diff-cover==11.0.3 apps/api/target/lcov.info --compare-branch origin/staging --include-untracked --fail-under 90`, e a mesma chamada com `--fail-under 100` e o `--include` dos caminhos críticos do `ci-api.yml` |

## `apps/web`

De `apps/web/`, depois do `bun install`. O CI roda os mesmos, no `ci-web.yml`.

| Gate | Comando |
| --- | --- |
| Vulnerabilidades conhecidas | `bun run audit` |
| Formato | `bunx prettier --check .` |
| Lint e fronteiras | `bun run lint` |
| Tipos dos arquivos de configuração e do e2e | `bun run typecheck` |
| Teste das fronteiras | `bun run test:boundaries` |
| Build | `bun run build` |
| Testes e cobertura | `bun run test -- --coverage --watch=false` |
| E2E, a 390 px e com o axe | `bun run e2e` |
| Linhas novas | da raiz: `uvx diff-cover==11.0.3 apps/web/coverage/web/lcov.info --compare-branch origin/staging --include-untracked --fail-under 90`, e a mesma chamada com `--fail-under 100` e o `--include` dos caminhos críticos do `ci-web.yml` |

## `apps/android`

De `apps/android/`, com o JDK 21 (`JAVA_HOME=$(/usr/libexec/java_home -v 21)` no macOS). O CI roda os
mesmos, menos os dois do emulador, no `ci-android.yml`. Todo comando do Gradle confere o SHA-256 de cada
artefato no `gradle/verification-metadata.xml` e as versões nos `gradle.lockfile`.

| Gate | Comando |
| --- | --- |
| Vulnerabilidades conhecidas | `osv-scanner scan source --lockfile app/gradle.lockfile --lockfile core/designsystem/gradle.lockfile`, com um `--lockfile` por módulo |
| Formato | `./gradlew spotlessCheck` |
| Análise estática | `./gradlew detekt` |
| Lint | `./gradlew lintDebug` |
| Fronteiras de módulo | `./gradlew :build-logic:convention:test` |
| Testes na JVM (Robolectric e Konsist) | `./gradlew testDebugUnitTest` |
| Cobertura | `./gradlew koverXmlReport` |
| Build | `./gradlew assembleDebug assembleDebugAndroidTest` |
| Linhas novas | da raiz: `uvx diff-cover==11.0.3 apps/android/build/reports/kover/report.xml --compare-branch origin/staging --include-untracked --src-roots <cada src/main/kotlin> --fail-under 90`, e a mesma chamada com `--fail-under 100` e o `--include` dos caminhos críticos do `ci-android.yml` |
| Acessibilidade, no emulador | `./gradlew connectedDebugAndroidTest` |
| E2E, no emulador | `./gradlew installDebug` e `maestro test e2e/` |

Dependência nova pede `./gradlew :app:dependencies :core:designsystem:dependencies --write-locks` (um
`:<módulo>:dependencies` por módulo) e depois `./gradlew --write-verification-metadata sha256` com os
gates acima, para o arquivo ver cada artefato.

## `apps/ios`

De `apps/ios/`, com o Xcode 27.2. O CI roda os mesmos no `ci-ios.yml`, no runner `xcode-27`. O SwiftLint
vem pelo command plugin do `SwiftLintPlugins`, travado no `Packages/Package.swift`; ninguém o instala.

| Gate | Comando |
| --- | --- |
| Formato | `xcrun swift-format lint --strict --recursive Clinicore ClinicoreUITests ArchitectureFixtures Packages/Package.swift Packages/Sources Packages/Tests` |
| Lint | `swift package --package-path Packages plugin --allow-writing-to-package-directory swiftlint lint --strict --quiet --config "$PWD/.swiftlint.yml" "$PWD/Clinicore" "$PWD/ClinicoreUITests" "$PWD/Packages/Package.swift" "$PWD/Packages/Sources" "$PWD/Packages/Tests"` |
| Fronteiras | o mesmo plugin com `--reporter json` sobre `"$PWD/ArchitectureFixtures"`: tem que reprovar, e a saída traz as quatro regras próprias, como o `ci-ios.yml` confere com o `jq` |
| Build e testes (Swift Testing e XCUITest) | `xcodebuild test -project Clinicore.xcodeproj -scheme Clinicore -destination 'platform=iOS Simulator,name=iPhone 17e,OS=27.2' -enableCodeCoverage YES -derivedDataPath build` |
| Cobertura | `xcrun llvm-cov export -format=lcov` sobre o `Coverage.profdata` em `build/Build/ProfileData/`, com `-object` no `Clinicore.app/Clinicore.debug.dylib` e no `DesignSystemTests.xctest/DesignSystemTests`, para `build/coverage.lcov` |
| Linhas novas | da raiz: `uvx diff-cover==11.0.3 apps/ios/build/coverage.lcov --compare-branch origin/staging --include-untracked --fail-under 90`, e a mesma chamada com `--fail-under 100` e o `--include` dos caminhos críticos do `ci-ios.yml` |

O `xcodebuild test` sobe o simulador; no Mac, ele roda sem Gradle nem emulador ao lado. Não há gate de
CVE: o osv-scanner não lê o `Package.resolved`, e o alerta vem do Dependabot (ADR 0024).

## `apps/site`

O `apps/site` ainda não tem código. Os gates dele entram nesta skill com o scaffold, como comandos que o
scaffold faz passar, e ele ganha o seu `ci-site.yml`.

| App | Gates | Ferramentas decididas |
| --- | --- | --- |
| `apps/site` | n/a: sem código | ESLint com `eslint-plugin-svelte` e `eslint-plugin-boundaries`, Prettier, `svelte-check`, `vite build`, Vitest (ADRs 0021, 0026) |

Em todo app, a cobertura das linhas novas é conferida pelo `diff-cover` contra a `origin/staging`, como a
skill `testing` descreve (ADR 0023).

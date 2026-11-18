# `apps/android` — forma e regras

Aditivo ao `AGENTS.md` da raiz. Stack na ADR 0022, forma na 0026, offline na 0007 (itens 3 a 9). App
nativo em Kotlin e Compose, com as telas do web e offline completo.

## Estrutura

```
apps/android/
├── gradle/libs.versions.toml  gradle/verification-metadata.xml
├── config/                  example.properties commitado; dev, staging e prod fora do git
├── build-logic/convention/  plugins clinicore.*: root, application, library, compose, e feature, jvm, room e hilt
│                            quando o primeiro módulo os usar; spotless, detekt, kover e as fronteiras valem em todo módulo
├── app/                     MainActivity, ClinicoreApplication, AppConfig.kt, NavDisplay e as abas
├── core/
│   ├── model/               JVM puro: o domínio (Patient, Appointment)
│   ├── common/              JVM puro: dispatchers injetáveis, Result, relógio
│   ├── security/            chaves no Keystore, AES-GCM e os apagamentos
│   ├── network/             Retrofit, OkHttp, Authenticator, ApiError, *Request, *Response
│   ├── database/            Room 3 + SQLCipher: ClinicalDatabase, OutboxDatabase, *Entity, *Dao; schemas/
│   ├── datastore/           TokenStore
│   ├── data/                os repositories: a única API pública dos dados
│   ├── sync/                WorkManager, esvaziamento da fila, Pendências
│   ├── designsystem/        tema Material 3 com os tokens, componentes base, ícones
│   ├── ui/                  componentes compostos de mais de uma feature
│   └── testing/             fakes dos repositories, MainDispatcherRule, fixtures de JSON
├── feature/<feature>/
│   ├── api/                 só as chaves de navegação (<Tela>NavKey)
│   └── impl/                navigation/<Feature>EntryProvider.kt e uma pasta por tela:
│                            <Tela>Screen.kt, <Tela>ViewModel.kt, <Tela>UiState.kt
└── e2e/<feature>/<fluxo>.yaml   Maestro
```

## Fronteiras

O Gradle impõe, pela tabela de `build-logic/convention/src/main/kotlin/ModuleBoundaries.kt`, que o
`ModuleBoundariesTest` prova linha por linha:

| Módulo | Depende de |
| --- | --- |
| `app` | as features e os `core` que usa |
| `feature:X:impl` | `core:*` (nunca `network`, `database` nem `datastore`) e o `api` de outra feature |
| `feature:X:api` | `core:model` |
| `core:data` | `network`, `database`, `datastore`, `model`, `common` |
| `core:*` | outro `core`, nunca feature |

Testes do Konsist conferem o resto: `ViewModel` só em `feature:*:impl`, com sufixo e `@HiltViewModel`;
`core:model` e `core:common` sem `android.*`; nenhum `*Dao`, `*Response` ou Retrofit numa feature;
`UiState` imutável. Cada regra também roda contra `app/src/test/konsist-fixtures/`, que a viola de
propósito.

## Padrões

- **A tela segue o guia de arquitetura do Android.** `<Tela>Route` fala com o ViewModel; `<Tela>Screen`
  é sem estado e recebe o state e os lambdas. O ViewModel expõe `val uiState: StateFlow<<Tela>UiState>`
  e uma função pública por evento, com nome de verbo. Camada de domínio só quando uma regra se repete.
- **O banco local é a fonte da verdade.** Toda escrita vai para a fila do `outbox.db` e sobe pelo
  WorkManager; recusa da API vira Pendência.
- **A navegação passa ID primitivo**; a tela carrega o dado do repository.
- **Tudo é `internal` por padrão**, e a dependência é `implementation`, não `api`.
- **JSON com `ignoreUnknownKeys` no app** e decodificação estrita sobre fixtures nos testes.
- **`AppConfig.kt` é o único leitor da configuração**, que o Gradle valida e põe no `BuildConfig`.
- **Teste na JVM primeiro**, com fakes; `androidTest` só onde ela não alcança. O ATF é um desses casos:
  no Robolectric ele não avalia Compose, então as checagens de acessibilidade rodam no emulador, e o
  contraste dos tokens é conferido na JVM, no `ContrastTest`.
- **Piso de segurança de dependência transitiva**: uma versão com vulnerabilidade conhecida que chega por
  ferramenta (lint, ATF, Robolectric) sobe por uma entrada `security-floor-*` no `libs.versions.toml`,
  aplicada como constraint em toda configuração. A constraint só sobe a versão e nunca acrescenta
  dependência.
- **Nada do app sai do aparelho por backup**: `allowBackup="false"`, e o `dataExtractionRules` e o
  `fullBackupContent` excluem todos os domínios.
- **Toda variante tem teste unitário** (`android.onlyEnableUnitTestForTheTestedBuildType=false`): sem
  isso, o sync de IDE do AGP 9.5 falha ao criar o `generateReleaseComposePreviewRunfiles`. A cobertura
  lê só a variante `unit` do Kover, que junta o `debug` e o `jvm`, então o gate não roda os testes do
  `release`.
- **O `verification-metadata.xml` confia nos `-sources.jar` e `-javadoc.jar`**, que a IDE baixa para
  mostrar e nunca executa; todo outro artefato tem SHA-256. Sem isso, o sync do Android Studio e do
  `kotlin-lsp` falha na verificação. O `aapt2` tem um jar por sistema, e o Gradle só registra o da
  máquina que gerou o arquivo: o de Linux, o de macOS e o de Windows entram à mão, com o SHA-256 do
  `dl.google.com`, a cada versão nova do AGP.

## Nomes

Pacote `br.com.clinicore.<caminho do módulo>`. Arquivo `.kt` em PascalCase com o nome do tipo principal
(regra do Kotlin, cobrada pelo detekt). `PatientsRepository` é a interface e
`OfflineFirstPatientsRepository` a implementação `internal`; `PatientEntity`, `PatientDao`,
`PatientResponse`; conversão por extensão, `fun PatientEntity.asModel()`.

## Caminhos críticos

100% das linhas novas (ADR 0023): `core/security`, `core/network`, `core/datastore`, `core/sync` e o
`OutboxDatabase` em `core/database`.

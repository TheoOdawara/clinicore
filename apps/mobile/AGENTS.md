# Clinicore `apps/mobile` — app nativo e pegadinhas

Aditivo ao `AGENTS.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este
arquivo apenas onde ela falar do mesmo assunto. A raiz tem a stack, os comandos, a visão geral da
arquitetura, as branches e o idioma.

## Camadas

- **`lib/main.dart` só sobe o app:** lê a configuração e entrega para o `appFor`.
- **`lib/app/`** tem o `MaterialApp` e o `GoRouter`. O `redirect` do `go_router` é o único guardião
  de rota autenticada.
- **`lib/features/<feature>/`** tem as telas, o estado e as mensagens de uma feature. Nenhuma feature
  cria o próprio `Dio`.
- **`lib/shared/env/env.dart` é o único leitor de configuração.** Ele lê por `String.fromEnvironment`,
  sem valor padrão, e devolve a configuração ou a lista de nomes faltantes. Faltando algum, o app sobe
  numa tela que só mostra "Missing configuration: <nomes>" e nenhum `Dio` é criado.
- **`lib/shared/http/http_client.dart` cria a instância única de `Dio`**, com `baseUrl` igual a `API_URL`
  e `Clinicore-Client: mobile` em toda requisição.
- **Nada é gerado do OpenAPI da API.** A chamada é o `Dio` direto, dentro de `features/<feature>/api/`,
  e o modelo da resposta é escrito à mão, com um `fromJson` que usa pattern matching e lança
  `FormatException` quando o payload foge do formato. É o papel do `.parse()` do Zod no web, e o DTO é
  replicado à mão do mesmo jeito.
- **O estado é Riverpod com o `riverpod_generator`**, que entra com a primeira tela; o `build_runner`
  entra junto, e o código que ele gera (`*.g.dart`) fica fora da cobertura.
- **Testes cobrem 100% de `lib/`**, fora o código gerado, com o `flutter_test`, o `mocktail` e o HTTP do
  `Dio` interceptado pelo `http_mock_adapter`. O e2e é o Patrol, em emulador Android e simulador iOS,
  contra a API real.
- **A URL é a da rota do `go_router`** e repete o caminho do web: `/login`, `/app/account/password`.

## Configuração

Os valores entram por `--dart-define-from-file=config/<ambiente>.json`. `config/example.json` é
commitado com o nome e o formato de cada valor, e o resto de `config/*.json` fica fora do git. No
emulador Android, a API local é `http://10.0.2.2:3333`, e o `AndroidManifest.xml` de `src/debug/`
libera HTTP sem TLS só no build de debug.

## Pegadinhas da stack

Verificadas em 2026-09-24 contra Flutter 3.47.5 e reconfirmadas em 2026-10-25 contra Flutter 3.47.8.

- **O `very_good_analysis` liga o `public_member_api_docs`**, que pede comentário de documentação em
  todo membro público. O contrato só aceita comentário `shortcut:`, então essa regra é a única desligada
  no `analysis_options.yaml`.
- **O `flutter test --coverage` não reprova abaixo de um limite.** Quem reprova é o `very_good_cli`,
  instalado com `dart pub global activate very_good_cli 1.5.0`, com `very_good test --coverage --min-coverage 100 --collect-coverage-from=all --test-randomize-ordering-seed random`.
- **Sem `--collect-coverage-from=all`, a cobertura só conta arquivo que algum teste importa.** Um arquivo
  de `lib/` sem teste nenhum some do relatório, e os 100% passam. Medido: o `main.dart` não aparecia no
  `lcov.info` até ganhar o próprio teste.
- **O primeiro `flutter build apk` baixa o NDK 28.2** pelo Gradle. No Windows, o `sdkmanager.bat` que o
  Flutter chama pode morrer com `0xC0000409`. Rodar `android/gradlew.bat :app:assembleDebug` uma vez
  instala o NDK pelo próprio Android Gradle Plugin, e o `flutter build` volta a passar.
- **`String.fromEnvironment` devolve `''` quando o nome falta.** Vazio conta como faltante, e isso é
  decidido no `parseConfiguration`, nunca no ponto de uso.

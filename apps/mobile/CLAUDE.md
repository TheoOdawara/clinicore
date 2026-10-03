# Clinicore `apps/mobile` — app nativo e pegadinhas

Aditivo ao `CLAUDE.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este
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
- **`lib/shared/api/` é o cliente gerado** do `/api-json` da API. Ele nunca é editado à mão e fica fora
  do `flutter analyze`.
- **A URL é a da rota do `go_router`** e repete o caminho do web: `/login`, `/app/account/password`.

## Configuração

Os valores entram por `--dart-define-from-file=config/<ambiente>.json`. `config/example.json` é
commitado com o nome e o formato de cada valor, e o resto de `config/*.json` fica fora do git. No
emulador Android, a API local é `http://10.0.2.2:3333`, e o `AndroidManifest.xml` de `src/debug/`
libera HTTP sem TLS só no build de debug.

## Regerar o cliente

Com a API de pé em `:3333`, de dentro do `apps/mobile`:

```
rm -rf lib/shared/api
dart run swagger_parser
dart run build_runner build
dart format lib/shared/api
```

O `swagger_parser` é configurado no próprio `pubspec.yaml`, com `merge_outputs`: o cliente inteiro sai em
`lib/shared/api/api.dart`, e o `json_serializable` escreve o `api.g.dart` ao lado. Rota nova na API só
chega ao app por essa regeração, e o diff do `lib/shared/api/` vai no mesmo pull request que a usa.

## Pegadinhas da stack

Verificadas em 2026-09-24, contra Flutter 3.47.5 e `swagger_parser` 1.44.3.

- **O `swagger_parser` não apaga o que deixou de existir.** Um schema renomeado na API deixa o arquivo
  antigo para trás, e é por isso que a receita começa apagando a pasta.
- **O código gerado não sai formatado.** Nem o `swagger_parser` nem o `build_runner` passam o
  `dart format`, e o gate `dart format --set-exit-if-changed .` reprova o cliente recém-gerado. Por
  isso o `dart format lib/shared/api` é o último passo da regeração.
- **O cliente gerado declara `Clinicore-Client` como parâmetro opcional em toda rota.** Omitido, o
  Retrofit não manda nada, e vale o header da instância de `Dio`. Um teste prova que a chamada gerada
  sai com o header uma vez só.
- **O primeiro `flutter build apk` baixa o NDK 28.2** pelo Gradle. No Windows, o `sdkmanager.bat` que o
  Flutter chama pode morrer com `0xC0000409`. Rodar `android/gradlew.bat :app:assembleDebug` uma vez
  instala o NDK pelo próprio Android Gradle Plugin, e o `flutter build` volta a passar.
- **`String.fromEnvironment` devolve `''` quando o nome falta.** Vazio conta como faltante, e isso é
  decidido no `parseConfiguration`, nunca no ponto de uso.

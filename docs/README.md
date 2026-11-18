# Documentação do Clinicore

| Pasta | O que tem |
| --- | --- |
| [`product/`](product/README.md) | o que o produto precisa ser: o SRS e o design |
| [`system/`](system/README.md) | como o sistema é construído: arquitetura e modelo de dados |
| [`decisions/`](decisions/README.md) | as ADRs: por que cada decisão transversal foi tomada |
| [`work/`](work/README.md) | o que o trabalho planejou e fez: roadmap, sprints e specs |

Não há `guides/`: o produto ainda não tem usuário que siga um guia. A pasta nasce com o primeiro.

As regras de como o código é construído não ficam aqui: estão no [`AGENTS.md`](https://github.com/TheoOdawara/clinicore/blob/staging/AGENTS.md) da raiz e
no de cada app.

## Ler como site

A mesma pasta vira um site local, com menu e busca:

```sh
uvx zensical==0.1.2 serve --open
```

Ele abre em `http://localhost:8000`. No Zed, é a task `docs: serve`. O site não é publicado em lugar
nenhum.

## Editar um diagrama

Cada diagrama é um `.drawio.svg`: um SVG que o GitHub e o site mostram, com o desenho editável dentro.
O Zed mostra o diagrama, mas não o edita: a edição é no draw.io desktop.

## Editor

Cada app abre como raiz própria, porque o servidor de linguagem só acha o projeto na raiz da pasta aberta:
o Gradle em `apps/android`, o Cargo em `apps/api` e o `node_modules` de cada app web. Aberto na raiz do
repo, o Gradle não é importado.

- **VS Code:** `code clinicore.code-workspace`. O workspace tem uma pasta por app e a pasta `repo`, com a
  documentação e os workflows. As extensões recomendadas estão em `.vscode/extensions.json`, a
  configuração de cada app em `apps/<app>/.vscode/`, e as tasks de docs e de gates no workspace.
- **Zed:** `zed -n apps/<app>`, ou `zed .` para a documentação. Sem o `-n`, com o repo já aberto, o
  Zed reaproveita a janela da raiz e o Gradle não é importado. A configuração de cada app está em
  `apps/<app>/.zed/`, com a task de gates dele. O Zed só instala extensão pela configuração global, então
  cada pessoa instala estas:

| Extensão | Para |
| --- | --- |
| `toml` | `Cargo.toml`, `deny.toml`, `zensical.toml` |
| `sql` | as migrations da API |
| `dockerfile` | o Dockerfile de cada app |
| `angular` | os templates do `apps/web` |
| `scss` | o estilo do `apps/web` |
| `svelte` | o `apps/site` |
| `kotlin` | o `apps/android`, pelo `kotlin-lsp` da JetBrains |
| `swift` | o `apps/ios`, pelo `sourcekit-lsp` |

O Rust, o TypeScript, o HTML, o CSS, o YAML e o Markdown já vêm no Zed. A extensão `java`, que o Zed
sugere, não serve ao projeto: o `apps/android/.zed/` desliga o servidor de Gradle dela e passa os
`.gradle.kts` ao `kotlin-lsp`.

**Android.** O Gradle roda o daemon no JDK 21 que o `gradle/gradle-daemon-jvm.properties` pede, sem
precisar de `JAVA_HOME`. Fora do Android Studio, o Kotlin funciona pelo `kotlin-lsp`, que ainda é alpha e
tem suporte experimental ao Android Gradle Plugin: completar e navegar funcionam no código do projeto,
mas o índice das bibliotecas do Android é incompleto. O código de tela é escrito no Android Studio.

**iOS.** O `sourcekit-lsp` não lê o `.xcodeproj`. A ponte é o `xcode-build-server`
(`brew install xcode-build-server`): depois do primeiro build do gate, a task `ios: build server` grava o
`apps/ios/buildServer.json`, que fica fora do git porque tem caminho absoluto. Um arquivo novo só ganha
completar depois do build seguinte.

O Android Studio e o Xcode continuam necessários para rodar os apps nativos no emulador e no simulador.

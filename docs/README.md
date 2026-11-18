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

O editor do projeto é o Zed. A configuração do projeto está em `.zed/`: o rust-analyzer com o clippy, o
formato ao salvar e as tasks de docs e de gates. O Zed só instala extensão pela configuração global, então
cada pessoa instala estas:

| Extensão | Para |
| --- | --- |
| `toml` | `Cargo.toml`, `deny.toml`, `zensical.toml` |
| `sql` | as migrations da API |
| `dockerfile` | o Dockerfile de cada app |
| `angular` | os templates do `apps/web` |
| `scss` | o estilo do `apps/web` |
| `svelte` | o `apps/site` |
| `kotlin` | o `apps/android` |
| `swift` | o `apps/ios` |

O Rust, o TypeScript, o HTML, o CSS, o YAML e o Markdown já vêm no Zed. O Android Studio e o Xcode
continuam necessários para rodar os apps nativos no emulador e no simulador.

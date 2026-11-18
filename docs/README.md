# Documentação do Clinicore

| Pasta | O que tem |
| --- | --- |
| [`requirements/`](requirements/README.md) | o SRS: problema, escopo e cada requisito com ID e critérios de aceite |
| [`architecture/`](architecture/README.md) | o mapa do sistema: blocos, cenários de runtime, implantação, conceitos e riscos |
| [`data-model/`](data-model/README.md) | as entidades, as relações e o armazenamento |
| [`design/`](design/README.md) | a ferramenta de design, a marca, os tokens e cada tela por feature |
| [`roadmap/`](roadmap/README.md) | os marcos divididos em épicos, a cobertura dos requisitos e o plano de cada sprint |
| [`decisions/`](decisions/README.md) | as ADRs: por que cada decisão transversal foi tomada |
| [`specs/`](specs/README.md) | uma entrega em profundidade |

As regras de como o código é construído não ficam aqui: estão no [`AGENTS.md`](https://github.com/TheoOdawara/clinicore/blob/staging/AGENTS.md) da raiz e
no de cada app.

## Ler como site

A mesma pasta vira um site local, com menu e busca:

```sh
uvx zensical@0.0.71 serve --open
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

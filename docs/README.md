# Documentação do Clinicore

| Pasta | O que tem |
| --- | --- |
| [`requirements/`](requirements/README.md) | o SRS: problema, escopo e cada requisito com ID e critérios de aceite |
| [`architecture/`](architecture/README.md) | o mapa do sistema: blocos, cenários de runtime, implantação, conceitos e riscos |
| [`data-model/`](data-model/README.md) | as entidades, as relações e o armazenamento |
| [`roadmap/`](roadmap/README.md) | os marcos divididos em épicos, a cobertura dos requisitos e o plano de cada sprint |
| [`decisions/`](decisions/README.md) | as ADRs: por que cada decisão transversal foi tomada |
| [`specs/`](specs/README.md) | uma entrega em profundidade |
| [`diagrams/`](diagrams/README.md) | os diagramas, em `.drawio.svg` |

As regras de como o código é construído não ficam aqui: estão no [`AGENTS.md`](../AGENTS.md) da raiz e
no de cada app.

## Ler como site

A mesma pasta vira um site local, com menu e busca:

```sh
uvx zensical@0.0.67 serve --open
```

Ele abre em `http://localhost:8000`. No VS Code, é a task `docs: serve`. O site não é publicado em lugar
nenhum.

## Editar um diagrama

Cada diagrama é um `.drawio.svg`: um SVG que o GitHub e o site mostram, com o desenho editável dentro.
No VS Code, a extensão `hediet.vscode-drawio` abre o arquivo direto no editor.

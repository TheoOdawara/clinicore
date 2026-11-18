# 0025. Scripts e ferramentas do repositório

- Status: accepted
- Date: 2026-11-17

## Context

O repositório não tem nenhum script, e o `AGENTS.md` deixou a linguagem do primeiro em aberto até ele ser
necessário. A revisão da stack mostrou que a cobertura e o CI se resolvem com ferramentas prontas, então
nenhum script nasce agora. A regra fica decidida antes do primeiro.

## Decision

1. **Um script de um app fica dentro dele e é escrito na linguagem dele**: Rust na API, TypeScript no web
   e no site, Kotlin no Android e Swift no iOS.
2. **Uma ferramenta que não pertence a nenhum app é escrita em Python** e fica em `tools/`, na raiz.
3. **A stack das ferramentas em Python**, com as versões de 2026-11-17:

   | Papel | Escolha | Versão |
   | --- | --- | --- |
   | Python | `requires-python = "==3.15.*"` no cabeçalho de cada script | 3.15.2 |
   | Gerenciador | uv, que baixa o Python e roda o script; no CI, a action `astral-sh/setup-uv` | 0.13.9 |
   | Forma | Um arquivo `.py` por ferramenta, com as dependências no cabeçalho (PEP 723) e o lock ao lado (`uv lock --script`); sem `pyproject.toml` na raiz | — |
   | Dependências | Biblioteca padrão primeiro; Pydantic e pydantic-settings em todo script; o resto só quando a biblioteca padrão não resolve, com `==` | 2.14.3 · 2.16.3 |
   | Dado externo | Todo JSON de fora vira modelo do Pydantic, nunca um `dict` solto | — |
   | Ambiente | `tools/shared/env.py` é o único leitor, por pydantic-settings | — |
   | Linha de comando | `argparse` | — |
   | GitHub | CLI `gh` por `subprocess` | — |
   | Lint e formato | Ruff, configurado em `tools/ruff.toml` | 0.17.5 |
   | Tipos | mypy `--strict` com o plugin do Pydantic, em `tools/mypy.ini` | 2.4.1 |
   | Testes | pytest em `tools/tests/` | 9.2.0 |
   | CI | `ci-tools.yml`, quando `tools/` muda | — |

4. **Cada script** tem tipos em tudo e um `main(argv) -> int`, escreve dado no stdout e mensagem no
   stderr pelo `logging`, e sai com 0 quando dá certo, 1 quando falha e 2 quando o uso está errado.
5. **Nenhum script `.sh`.** Shell só como comando de uma linha no YAML do CI; o que passar disso vira
   Python.

## Consequences

- **A pasta `tools/` nasce com a primeira ferramenta**, não antes.
- **O Python entra no repositório como quinta linguagem**, mas só para ferramenta, nunca para código de
  produto.

## Alternatives considered

- **pip** · rejeitado: não instala a versão do Python, e o `pip lock` ainda é experimental e só vale
  para a versão do Python e a plataforma em que foi gerado.
- **ty, o checador de tipos da Astral** · rejeitado por enquanto: está em beta.
- **Bash** · rejeitado: ler JSON e tratar erro fica frágil.
- **TypeScript no Bun para os scripts do repositório** · rejeitado: poria o Bun nos jobs do Android e do
  iOS só para isso.

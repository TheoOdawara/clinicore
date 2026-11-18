---
name: testing
description: Como escrever e revisar teste no Clinicore, inclusive teste escrito por agente, e como a cobertura é medida. Use ao escrever, mudar ou revisar um teste em qualquer app.
---

# testing

## O que um teste precisa ter

- **Afirma um comportamento que alguém observa**: o status e o corpo da resposta, o estado da tela, a
  linha no banco, o e-mail na fila. Nunca a chamada interna de uma função.
- **Nunca usa mock da peça que está testando.** Fake ou stub entra só na fronteira: a rede, o relógio, o
  sistema operacional.
- **Tem asserção que falharia se o código estivesse errado.** Teste que passa com o corpo da função
  apagado é reprovado na revisão, mesmo cobrindo a linha.
- **Um comportamento por teste**, com o nome dizendo qual (`refresh_with_a_revoked_session_is_refused`).
- **Não depende de ordem nem de relógio real.** Hora é injetada; dado é gerado por teste.

## Cobertura (ADR 0023)

- **90% das linhas novas ou alteradas de cada PR**, medidas pelo `diff-cover` contra a `origin/staging`.
- **100% das linhas novas nos caminhos críticos** de cada app, listados no `AGENTS.md` dele, numa segunda
  chamada do `diff-cover` com `--include`.
- **Ficam fora da conta**, declarados no gerador de cobertura de cada app: código gerado, o ponto de
  entrada e as telas, que são cobertas pelo teste de interface.
- Não há piso global. Um teste apagado aparece no diff do PR.

| App | Relatório que o `diff-cover` lê |
| --- | --- |
| `apps/api` | lcov do `cargo llvm-cov` |
| `apps/web` e `apps/site` | lcov do Vitest |
| `apps/android` | XML JaCoCo do Kover |
| `apps/ios` | lcov do `llvm-cov export` sobre o perfil do Xcode |

Os comandos estão na skill `gates`.

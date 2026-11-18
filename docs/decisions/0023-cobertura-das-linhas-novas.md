# 0023. Cobertura medida nas linhas novas de cada PR

- Status: accepted
- Date: 2026-11-17
- Substitui: 0019 na exigência de 100% de cobertura em cada app

## Context

A 0019 exigiu 100% de cobertura em cada app. O dono do produto quer entregar mais rápido e com o mínimo
de erro, e duvidou que o número inteiro fosse o caminho.

O que pesou:

- O Google trata 60% como aceitável, 75% como louvável e 90% como exemplar, diz que meta acima de 90% no
  projeto inteiro provavelmente não compensa, e que 99% por commit é razoável, com 90% como piso
  ([Google Testing Blog, 2020](https://testing.googleblog.com/2020/08/code-coverage-best-practices.html)).
- Martin Fowler desconfia de qualquer coisa perto de 100%, porque o número vira alvo e aparece teste
  escrito para ele ([Test Coverage](https://martinfowler.com/bliki/TestCoverage.html)).
- Com agente escrevendo código, cobertura é o alvo mais fácil de cumprir com teste que não confere nada:
  há relato de suíte com 100% de linhas e 4% de mutantes mortos.

Os apps web, Android e iOS ainda vão ser escritos, e a API está em 100%. Cobrir o que entra em cada PR
mantém o total alto sem custo em código antigo.

## Decision

1. **Cada PR reprova se menos de 90% das linhas novas ou alteradas tiverem teste.** O `diff-cover`
   11.0.3 compara o PR com a `staging` sobre o relatório de cobertura de cada app, chamado direto do
   workflow com `uvx diff-cover==11.0.3`.
2. **Nos caminhos críticos, a exigência sobe para 100% das linhas novas**, numa segunda chamada do
   `diff-cover` restrita a eles por `--include`. São críticos:
   - autenticação, sessão e tokens;
   - fila offline e sincronização;
   - cifragem e os apagamentos (72 horas, logout e revogação);
   - assinatura digital (0013).

   Os caminhos exatos de cada app ficam no `AGENTS.md` dele.
3. **Ficam fora da conta**, declarados uma vez por app: o código gerado, o ponto de entrada do app e as
   telas, que são testadas pela interface.
4. **Cada app gera um relatório que o `diff-cover` lê**:

   | App | Relatório |
   | --- | --- |
   | `apps/api` | lcov do `cargo llvm-cov` |
   | `apps/web` e `apps/site` | lcov do Vitest |
   | `apps/android` | XML JaCoCo do Kover |
   | `apps/ios` | lcov do `llvm-cov export` sobre o perfil do Xcode |

## Consequences

- **Não há piso global.** Com toda linha nova coberta, o total não cai. Sobra o caso de um teste
  apagado, que aparece no diff do PR.
- **Nenhum script nasce para isso.** O `diff-cover` é a ferramenta inteira.
- **O teste de mutação fica para depois.** Ele é o que mede se o teste confere alguma coisa, e entra se os
  testes escritos por agente começarem a parecer vazios. As ferramentas existem para a API
  (cargo-mutants), para o web e o site (StrykerJS) e para o Android (Pitest, que no Kotlin precisa de um
  plugin pago); não há uma viável para o iOS.
- **A skill de testes ganha a regra para teste escrito por agente:** todo teste afirma um comportamento
  observável, nunca usa mock da própria peça que testa, e teste sem asserção relevante é reprovado na
  revisão.

## Alternatives considered

- **Manter 100% em cada app (0019)** · rejeitada: custa onde o risco é baixo e incentiva teste vazio.
- **100% das linhas novas com exclusões** · rejeitada pelo dono do produto, em favor de 90% com os
  caminhos críticos em 100%.
- **Piso global de 90% por unidade** · rejeitado: redundante com a cobertura das linhas novas.
- **Teste de mutação semanal desde já** · adiado: custo fixo antes de existir o problema.

# Sprint 2 — 2026-10-19 → 2026-11-01

**Meta:** uma pessoa cria a conta, confirma o e-mail, entra com senha ou com Google, recupera a senha e
sai, pelo navegador.

**Baseline dos requisitos:** v2.0.0

A meta é a da [Sprint 1](sprint-01.md), que não chegou nela. Nenhuma `requirement-change` ficou aberta:
a #151, que leva o SRS à v2.0.0, já está no #155.

**Itens:** #151, #152, #153, #156, #119, #120, #150, #146, #148, #149, #147, #130, #133, #71, #131, #127,
#135, #128, #126, #137, #136, #154, #77, #78, #79, #80

## Ordem

| # | Item | Depende de |
| --- | --- | --- |
| 1 | #151, #152 e #153 — SRS v2.0.0, ADR do deploy e realinhamento do repo, no #155 | — |
| 2 | #156 — rustls 0.23.46, no #157 | — |
| 3 | #146 — recusar a sessão aberta com uma senha que o reset trocou | — |
| 4 | #148 — vincular o Google automaticamente só em endereço de que ele é autoridade | — |
| 5 | #149 — os blocos `oauth` e `password_resets` no diagrama de componentes da API | — |
| 6 | #119, #120 e #150 — reset de senha e Google na API, num pull request só com a #146, a #148 e a #149 | #146, #148, #149 |
| 7 | #147 — trocar o yoke-derive 0.8.3, retirado do registry | — |
| 8 | #130 e #133 — sequestro de conta no cadastro repetido e reserva de envio atômica | #119 |
| 9 | #71 — o worker, com a fila de e-mail e a purga diária, no lugar da #132 | #133 |
| 10 | #131, #127 e #135 — IP ilegível, validação de e-mail e invariantes no schema | — |
| 11 | #128, #126 e #137 — testes, limites no OpenAPI com o gate do client Dart, e arrumação | — |
| 12 | #136 — a cota por endereço conta só o e-mail que sai | #133, #71 |
| 13 | #154 — web e site sobre Bun | — |
| 14 | #77 — CSS Modules no lugar do Tailwind, no web e no site | #154 |
| 15 | #78 — telas de entrar e de criar conta | #77 |
| 16 | #79 — área protegida, renovação da sessão e sair | #78 |
| 17 | #80 — recuperar, redefinir e trocar a senha no web | #119, #79 |

A #146, a #148 e a #149 corrigem o que a #119 e a #120 construíram, então entram na mesma branch antes do
pull request. A #154 vem antes da #77 e da #78 para que os primeiros testes de componente já nasçam em
`bun test`.

O restante da auditoria do cadastro (#124) entrou na sprint em 2026-10-20, antes do web. São problemas
da API recém-escrita, e a triagem fechou a #125 e a #129, resolvidas, e a #134, obsoleta.

A #71 voltou para a sprint em 2026-10-20 e ocupa o lugar da #132. O envio de e-mail sai da API para o
worker, então o semáforo e o `TaskTracker` da #132 perdem onde morar, e o `pool` e o `timeout` do
`lettre` entram na #71. Ela vem depois da #133 porque o e-mail entra na fila na mesma transação do token,
e antes da #136 porque a cota passa a contar a linha da fila.

## Specs

- **#154:** a [ADR 0015](../../decisions/0015-front-em-bun.md) decide o que ela faz. Não há spec de
  feature, porque nada muda para quem usa.
- **#77:** a [ADR 0016](../../decisions/0016-front-em-css-modules.md).
- **#71:** [006 · worker e fila de e-mail](../../specs/autenticacao/006-worker-e-fila-de-email-api.md).
- **As demais:** fechadas em [003 · API](../../specs/autenticacao/003-autenticacao-api.md),
  [003 · web](../../specs/autenticacao/003-autenticacao-web.md) e
  [005](../../specs/005-migrar-api-para-rust.md).

## Fora desta sprint

O web instalável (#81), as telas do app (#96 a #99) e a passkey.

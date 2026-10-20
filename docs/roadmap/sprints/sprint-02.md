# Sprint 2 — 2026-10-19 → 2026-11-01

**Meta:** uma pessoa cria a conta, confirma o e-mail, entra com senha ou com Google, recupera a senha e
sai, pelo navegador.

**Baseline dos requisitos:** v2.0.0

A meta é a da [Sprint 1](sprint-01.md), que não chegou nela. Nenhuma `requirement-change` ficou aberta:
a #151, que leva o SRS à v2.0.0, já está no #155.

**Itens:** #151, #152, #153, #156, #119, #120, #150, #146, #148, #149, #147, #154, #77, #130, #78, #79,
#80

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
| 8 | #154 — web e site sobre Bun | — |
| 9 | #77 — CSS Modules no lugar do Tailwind, no web e no site | #154 |
| 10 | #130 — impedir o sequestro de conta no cadastro repetido de e-mail não verificado | — |
| 11 | #78 — telas de entrar e de criar conta | #77 |
| 12 | #79 — área protegida, renovação da sessão e sair | #78 |
| 13 | #80 — recuperar, redefinir e trocar a senha no web | #119, #79 |

A #146, a #148 e a #149 corrigem o que a #119 e a #120 construíram, então entram na mesma branch antes do
pull request. A #154 vem antes da #77 e da #78 para que os primeiros testes de componente já nasçam em
`bun test`.

## Specs

- **#154:** a [ADR 0015](../../decisions/0015-front-em-bun.md) decide o que ela faz. Não há spec de
  feature, porque nada muda para quem usa.
- **#77:** a [ADR 0016](../../decisions/0016-front-em-css-modules.md).
- **As demais:** fechadas em [003 · API](../../specs/autenticacao/003-autenticacao-api.md),
  [003 · web](../../specs/autenticacao/003-autenticacao-web.md) e
  [005](../../specs/005-migrar-api-para-rust.md).

## Fora desta sprint

O web instalável (#81), as telas do app (#96 a #99), a passkey e o restante da auditoria do cadastro
(#124).

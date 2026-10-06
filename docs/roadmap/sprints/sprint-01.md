# Sprint 1 — 2026-10-05 → 2026-10-18

**Meta:** uma pessoa cria a conta, confirma o e-mail, entra com senha ou com Google, recupera a senha e
sai, pelo navegador.

**Baseline dos requisitos:** v1.0.0

**Itens:** #119, #120, #130, #77, #78, #79, #80

Todos são do épico #3, e as specs já estão fechadas:
[003 · API](../../specs/autenticacao/003-autenticacao-api.md),
[003 · web](../../specs/autenticacao/003-autenticacao-web.md) e
[005](../../specs/005-migrar-api-para-rust.md).

## Ordem

| # | Item | Depende de |
| --- | --- | --- |
| 1 | #119 — recuperar e trocar a senha na API | — |
| 2 | #120 — entrar com Google na API | — |
| 3 | #130 — impedir o sequestro de conta no cadastro repetido de e-mail não verificado | — |
| 4 | #77 — shadcn/ui como design system do web | — |
| 5 | #78 — telas de entrar e de criar conta | #77 |
| 6 | #79 — área protegida, renovação da sessão e sair | #78 |
| 7 | #80 — recuperar, redefinir e trocar a senha no web | #119, #79 |

## Fora desta sprint

O web instalável (#81), as telas do app (#96 a #99), a passkey e o restante da auditoria do cadastro
(#124).

# Sprint 1 — 2026-11-17 → 2026-11-30

**Meta:** uma pessoa cria a conta, confirma o e-mail, entra com senha ou com Google, recupera a senha e
sai no web, no Android e no iOS, contra uma API sem dívida de segurança conhecida.

**Baseline dos requisitos:** v2.0.2. Nenhuma `requirement-change` ficou aberta: a #167 levou o SRS à
v2.0.2 e a #163, à v2.0.1.

**Estimativa:** 92 pontos, para 10 dias de uma pessoa. Não há teto: é a primeira sprint com o campo
`Estimate`, e o teto sai da média desta e das próximas.

A numeração recomeça aqui. As sprints anteriores, de 2026-10-05 a 2026-11-01, foram planejadas para a
API em NestJS e o app em Flutter, e o histórico delas está no git. As issues de domínio #6 a #56 foram
fechadas: cada épico é recriado a partir do `docs/requirements/` quando entrar numa sprint, com uma issue
por app.

## Itens

O épico é a #3. A revisão da API (#180) é épico das próprias correções.

| # | Item | Pontos | Depende de |
| --- | --- | --- | --- |
| 1 | Release da `staging` para a `main`, que fecha #114, #119, #120, #143, #150 a #153, #156, #159 e #163 | — | — |
| 2 | PR da `feature/167-workflow-setup` para a `staging`, com #71, #109, #133, #147, #154, #161 e #167 | — | — |
| 3 | #165 — docs alinhados à stack atual, com as specs 003-web, 004-mobile e 004-api reescritas | 3 | 2 |
| 4 | #166 — migrations aplicadas no boot da API | 1 | 2 |
| 5 | #146, #148, #130, #131 e #136 — dívida de segurança da autenticação na API | 2 · 2 · 3 · 2 · 2 | 2 |
| 6 | #180 — revisão completa da API | 5 | 5 |
| 7 | #126, #127, #128, #135, #137 e #149 — achados anteriores, sub-issues da #180 | 1 · 1 · 3 · 2 · 1 · 1 | 6 |
| 8 | #181 — padrão de comentários do repositório | 2 | — |
| 9 | #94 — sessão por ID token do Google para o app | 3 | 3, 5 |
| 10 | #169 — scaffold do web em Angular | 5 | 3 |
| 11 | #78, #79 e #80 — telas de autenticação do web | 5 · 5 · 3 | 10 |
| 12 | #170 — scaffold do Android | 5 | 3 |
| 13 | #172, #174 e #176 — gêmeas Android de #78, #79 e #80 | 5 · 5 · 3 | 12 |
| 14 | #178 — Google nativo no Android | 2 | 9, 13 |
| 15 | #171 — scaffold do iOS | 5 | 3 |
| 16 | #173, #175 e #177 — gêmeas iOS de #78, #79 e #80 | 5 · 5 · 3 | 15 |
| 17 | #179 — Google nativo no iOS | 2 | 9, 16 |

Os itens 1 e 2 levam trabalho feito antes desta sprint e não contam pontos. Os itens em `Staging` ficam
no Project até o release fechá-los.

A #180 tem 5 pontos pela auditoria em si. Cada achado confirmado vira sub-issue dela, estimada quando
aparece, e entra nesta sprint; a soma desses achados é registrada no fechamento.

## Specs

As telas não começam antes das specs. São o primeiro trabalho da sprint, dentro da #165, cada uma por
`/tightship:spec` e depois `/tightship:epic`, com as tasks penduradas na #3:

- [003 · web](../../specs/autenticacao/003-autenticacao-web.md), reescrita para Angular (ADR 0020);
- [004 · app](../../specs/autenticacao/004-autenticacao-mobile.md), reescrita para Kotlin e Swift
  (ADR 0022), sem passkey;
- [004 · API](../../specs/autenticacao/004-sessao-por-token-api.md), reescrita para a API em Rust, sem
  passkey, com a rota da #94.

A API de autenticação já tem spec fechada: [003 · API](../../specs/autenticacao/003-autenticacao-api.md)
e [006](../../specs/autenticacao/006-worker-e-fila-de-email-api.md).

## Fora desta sprint

O ambiente de homolog (#4), o web instalável (#81), o design system (#162), a escolha do nome (#168), a
passkey (#90, #92 e #100 a #102) e o offline do app (#144).

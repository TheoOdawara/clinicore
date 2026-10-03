# Eficiência de desempenho — `PERF`

Quanto tempo o sistema leva para responder.

## NFR-PERF-01 — Tempo de resposta da API

O sistema deve responder a 95% das requisições da API em até 500 ms, medidos no servidor ao longo de um mês de produção.

| Atributo | Valor |
| --- | --- |
| Justificativa | Tela lenta é uma das quatro queixas contra o legado. |
| Origem | Theo, elicitação de 2026-10-03 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-PERF-01.1** — O percentil 95 da duração das requisições da API, num mês de produção, é menor ou igual a 500 ms.

## NFR-PERF-02 — Tempo de exibição da tela

O sistema deve exibir o conteúdo principal de cada tela do web em até 2,5 s numa conexão 4G.

| Atributo | Valor |
| --- | --- |
| Justificativa | O tempo que a pessoa percebe é o da tela, não o da API. |
| Origem | Theo, elicitação de 2026-10-03 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-PERF-02.1** — O Largest Contentful Paint de cada tela do web, medido em perfil de rede 4G, é menor ou igual a 2,5 s.

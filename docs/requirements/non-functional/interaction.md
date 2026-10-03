# Capacidade de interação — `INTR`

Como a pessoa enxerga, alcança e entende a interface.

## NFR-INTR-01 — Mobile-first

O sistema deve apresentar toda tela utilizável na largura de 390 px, sem rolagem horizontal.

| Atributo | Valor |
| --- | --- |
| Justificativa | UX simples no celular é o principal argumento contra o legado, pensado para desktop. |
| Origem | Theo, kickoff de 2026-09-15 · largura de referência do contrato de desenvolvimento do Theo |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-INTR-01.1** — Em cada tela, na largura de 390 px, toda ação da tela é alcançável e a página não rola na horizontal.

## NFR-INTR-02 — Acessibilidade

O sistema deve atender aos critérios de sucesso do WCAG 2.2 no nível AA em toda tela do web e do app.

| Atributo | Valor |
| --- | --- |
| Justificativa | Contraste, navegação por teclado e leitor de tela são exigidos. |
| Origem | Theo, elicitação de 2026-10-03 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-INTR-02.1** — Cada tela passa em todos os critérios de sucesso de nível A e AA do WCAG 2.2.

## NFR-INTR-03 — Interface em pt-BR

O sistema deve apresentar todo texto da interface em português do Brasil.

| Atributo | Valor |
| --- | --- |
| Justificativa | O produto atende clínicas no Brasil. |
| Origem | Theo, kickoff de 2026-09-15 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**
- **NFR-INTR-03.1** — Nenhuma tela exibe texto de interface em outro idioma: 0 ocorrências.

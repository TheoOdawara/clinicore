# Flexibilidade — `FLEX`

Em que plataformas o sistema roda e como é instalado.

## NFR-FLEX-01 — Web e app nativo com as mesmas telas

O sistema deve oferecer as mesmas telas no web responsivo e no app nativo de iOS e de Android.

| Atributo | Valor |
| --- | --- |
| Justificativa | Web e app entram juntos no MVP e no go-live da piloto. |
| Origem | [ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md) |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**

- **NFR-FLEX-01.1** — 100% das telas do web existem no app de iOS e no de Android.

## NFR-FLEX-02 — Web instalável

O sistema deve permitir instalar o web como aplicativo no dispositivo.

| Atributo | Valor |
| --- | --- |
| Justificativa | O web é aberto como aplicativo, sem a barra do navegador. |
| Origem | Theo, ao especificar a autenticação: [spec 003 — web](../../specs/autenticacao/003-autenticacao-web.md), regra 9 |
| Prioridade | Must |
| Status | approved |
| Marco | M1 |
| Desde | v1.0.0 |

**Critérios de aceite**

- **NFR-FLEX-02.1** — O navegador oferece a instalação do web e o manifesto não tem erro.

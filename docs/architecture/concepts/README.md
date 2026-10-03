# Conceitos transversais

O que atravessa mais de um bloco e precisa ser igual em todos.

| Conceito | O que decide | Estado |
| --- | --- | --- |
| [Segurança](security.md) | autenticação, sessão, isolamento entre clientes e dado de saúde | sessão existe; isolamento e papéis no M1 |
| [Contrato de erro](error-contract.md) | o formato de todo erro da API e quem o traduz para a pessoa | existe |
| [Configuração e segredos](configuration.md) | como cada app recebe e valida a configuração | existe |
| [Offline](offline.md) | o que o app guarda no aparelho e como uma escrita offline chega à API | M1 |

Log não tem arquivo próprio: só a API registra log hoje, e a regra está no
[`apps/api/AGENTS.md`](../../../apps/api/AGENTS.md), em Log.

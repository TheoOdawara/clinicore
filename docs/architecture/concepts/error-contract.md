# Contrato de erro

Todo erro da API é um Problem Details da RFC 9457, em `application/problem+json`
([ADR 0006](../../decisions/0006-api-rest-e-problem-details.md)).

## Na API

- **Um lugar só monta a resposta de erro**: `crates/app/src/http/error.rs`. Cada feature tem o próprio
  erro tipado e o converte para o `AppError`.
- **Um erro com código tem status, slug e título**, e o `type` é `tag:clinicore.com.br,2026:<slug>`.
- **O erro do protocolo é `about:blank`**: rota inexistente, método não aceito, corpo malformado ou
  grande demais.
- **Falha interna sai sem detalhe**: `500` para banco e panic, `503` para o Redis fora. A causa vai só
  para o log.
- **Erro de validação traz um `code` por campo**, o do validador que recusou.

## Nos clientes — M1

As telas que consomem erro ainda não existem; estas são as regras com que elas nascem.

- **O texto que a pessoa lê é pt-BR e mora no cliente**, traduzido a partir do slug e do `code`. A API
  não manda mensagem para exibir.
- **Código sem tradução cai numa mensagem genérica**, nunca no texto cru da API.
- **O `401` no web dispara o refresh uma vez**, e as requisições concorrentes esperam por ele.

As regras de implementação estão no [`apps/api/AGENTS.md`](../../../apps/api/AGENTS.md) e no
[`apps/web/AGENTS.md`](../../../apps/web/AGENTS.md).

# Decisões de arquitetura

Uma ADR por decisão transversal: o contexto, a escolha, as consequências e o que foi descartado. Uma ADR
aceita não é reescrita; outra a emenda ou a substitui.

| ADR | Decisão | Status |
| --- | --- | --- |
| [0001](0001-api-em-nestjs-typeorm-e-redis.md) | API em NestJS, TypeORM e Redis | substituída pela 0009 |
| [0002](0002-web-e-site-em-vite.md) | Web e site em Vite, com prerender no site | aceita, emendada pela 0004 e pela 0007 |
| [0003](0003-design-system-com-shadcn-ui-e-react-bits.md) | Design system com shadcn/ui, e React Bits com escopo por categoria | aceita; voltou a valer inteira pela 0018, emendada pela 0019 na animação |
| [0004](0004-lint-do-front-em-eslint.md) | Lint do front em ESLint, com TypeScript 6 | substituída pela 0019 |
| [0005](0005-front-em-npm-sobre-node.md) | Front em npm sobre Node 26 | substituída pela 0015, e esta pela 0019 |
| [0006](0006-api-rest-e-problem-details.md) | API em REST orientado a recurso, com erro em Problem Details | aceita |
| [0007](0007-app-nativo-em-flutter-com-offline.md) | App nativo em Flutter, com offline completo | aceita, emendada pela spec `004-sessao-por-token-api.md` e pela 0019 |
| [0008](0008-site-em-next-standalone.md) | Páginas públicas em Next.js, com servidor standalone | aceita, emendada pela 0014 |
| [0009](0009-api-em-rust-com-axum-e-sqlx.md) | API em Rust, com axum e sqlx | aceita na stack; substituída pela 0010 na organização do workspace |
| [0010](0010-api-rs-com-crate-por-processo.md) | API em Rust com um crate por processo e camada só quando tem conteúdo | aceita |
| [0011](0011-openapi-da-api-rs-nasce-com-a-rota.md) | O OpenAPI da API nasce com a rota | aceita, emendada pela 0019 |
| [0012](0012-revogacao-de-sessao-antes-de-apagar.md) | Revogação de sessão antes de apagar | aceita |
| [0013](0013-assinatura-digital-em-duas-camadas.md) | Assinatura digital em duas camadas, por provedor existente | aceita |
| [0014](0014-deploy-por-dockerfile-no-coolify.md) | Deploy de cada app pelo próprio Dockerfile no Coolify | aceita, emendada pela 0019 |
| [0015](0015-front-em-bun.md) | Front em Bun, como gerenciador de pacotes e runtime | substituída pela 0019 |
| [0016](0016-front-em-css-modules.md) | Front em CSS puro, com CSS Modules | substituída pela 0018 |
| [0017](0017-fila-de-email-no-postgres-e-worker-com-cron.md) | Fila de e-mail numa tabela do Postgres e worker com cron | aceita |
| [0018](0018-front-volta-ao-shadcn-ui.md) | Front volta ao Tailwind e ao shadcn/ui | aceita |
| [0019](0019-revisao-da-stack-dos-quatro-apps.md) | Revisão da stack dos quatro apps: versões, gates, cobertura de 100% e mobile sem cliente gerado | aceita |

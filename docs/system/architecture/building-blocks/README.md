# Blocos de construção

O sistema decomposto em processos e armazenamentos. Cada bloco tem o próprio arquivo, com o que ele
faz, o que ele nunca faz e o que é preciso saber para mudá-lo.

![Diagrama de containers](../diagrams/containers.drawio.svg)

| Bloco | Tecnologia | Responsabilidade | Fala com | Marco | Arquivo |
| --- | --- | --- | --- | --- | --- |
| `apps/api` | Rust · axum · sqlx | o contrato HTTP e toda a regra de negócio | PostgreSQL (SQL) e Redis (RESP) | M1 | [api.md](api.md) |
| `apps/web` | Angular · SPA estático com PWA | o sistema da clínica no navegador | `apps/api` (HTTPS, JSON, sessão em cookie) | M1 | [web.md](web.md) |
| `apps/android` | Kotlin · Jetpack Compose | o sistema da clínica no Android, com offline | `apps/api` (HTTPS, JSON, token no `Authorization`) | M1 | [android.md](android.md) |
| `apps/ios` | Swift · SwiftUI | o sistema da clínica no iOS, com offline | `apps/api` (HTTPS, JSON, token no `Authorization`) | M1 | [ios.md](ios.md) |
| `apps/site` | SvelteKit estático | a landing pública e indexável | o `apps/web`, por link | M3 | [site.md](site.md) |
| PostgreSQL 18 | banco relacional | todo dado durável | — | M1 | [postgres.md](postgres.md) |
| Redis 8 | chave-valor com AOF | contagem do limite de requisições e denylist de sessão | — | M1 | [redis.md](redis.md) |
| worker | Rust · tokio · croner · lettre, em `apps/api/crates/worker/` | a purga diária do que venceu e o envio do e-mail da fila `email_outbox`, com retry | PostgreSQL (SQL) e SMTP | M1 | [worker.md](worker.md) |
| Banco local dos apps | Room com SQLCipher no Android; SwiftData sob Data Protection no iOS | o dado offline e a fila de escrita | — | M1 · [ADR 0022](../../../decisions/0022-apps-nativos-em-kotlin-e-swift.md) | descrito em [android.md](android.md) e [ios.md](ios.md) |

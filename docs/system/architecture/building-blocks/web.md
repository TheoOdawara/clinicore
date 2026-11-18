# `apps/web`

O sistema da clínica no navegador, em `app.clinicore.com.br`: um SPA estático em Angular, instalável
como PWA e servido pelo nginx, só para usuário autenticado
([ADR 0020](../../../decisions/0020-web-em-angular.md)). Fala direto com a API, em origem cruzada, com a
sessão em cookie.

**Nunca faz:** guardar dado clínico offline (isso é só dos apps nativos, pela
[ADR 0007](../../../decisions/0007-app-nativo-em-flutter-com-offline.md)), importar código de outro app,
ter regra de negócio que a API não tenha.

O app ainda não existe; nasce no M1.

## Blocos internos

| Bloco | Onde | Responsabilidade | Estado |
| --- | --- | --- | --- |
| Shell | `src/app/shell/` | o layout autenticado: barra, menu, conteúdo | M1 |
| Configuração | `src/app/core/config/` | lê e valida os valores do build | M1 |
| Cliente HTTP | `src/app/core/http/` | os interceptors de credencial e de refresh, e o Problem Details | M1 |
| Sessão | `src/app/core/session/` | o usuário atual e o guard da área autenticada | M1 |
| Locale | `src/app/core/locale/` | pt-BR no Angular e no Material | M1 |
| UI genérica | `src/app/shared/` | componentes de mais de uma feature, sem estado | M1 |
| Features | `src/app/features/<feature>/` | `data/` (API, schema Zod, store), `pages/` e `ui/` de uma feature | M1, a começar pela autenticação |
| Tema | `src/styles/` | o `mat.theme` sobre os tokens de `docs/product/design/` | M1 |
| PWA instalável | manifesto e service worker | instalação pelo navegador, sem guardar resposta da API | M1 |

## Para mudar com segurança

As fronteiras entre as pastas, conferidas por lint, e os nomes estão no
[`apps/web/AGENTS.md`](https://github.com/TheoOdawara/clinicore/blob/staging/apps/web/AGENTS.md). O schema Zod de cada resposta replica à mão
o que a API devolve, então uma mudança de contrato na API pede a mudança correspondente aqui.

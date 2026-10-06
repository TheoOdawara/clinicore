# `apps/web`

O sistema da clínica no navegador, em `app.clinicore.com.br`: uma SPA em React 19 com Vite e TanStack
Router, só para usuário autenticado. Fala direto com a API, em origem cruzada, com a sessão em cookie.

**Nunca faz:** guardar dado clínico offline (isso é só do app, pela
[ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md)), importar código de outro app,
ter regra de negócio que a API não tenha.

## Blocos internos

| Bloco | Onde | Responsabilidade | Estado |
| --- | --- | --- | --- |
| Rotas | `src/routes/` | caminho, `loader` e composição das features | existe, com a rota raiz |
| Configuração | `src/shared/env/` | lê e valida as variáveis do build | existe |
| Estilo | `src/styles/` | Tailwind e os tokens | existe |
| Cliente HTTP | `src/shared/http/` | a instância única do axios, com o refresh da sessão no interceptor | M1 · #79 |
| UI base | `src/shared/` | os componentes do shadcn/ui | M1 · #77 |
| Features | `src/features/<feature>/` | chamada à API, schema Zod da resposta, componentes e hooks de uma feature | M1, a começar pela autenticação (#78) |
| PWA instalável | manifesto e service worker | instalação pelo navegador, sem cachear a origem da API | M1 · #81 |

## Para mudar com segurança

A direção das dependências (`shared → features → routes`) e as regras de cada pasta estão no
[`apps/web/AGENTS.md`](../../../apps/web/AGENTS.md). O schema Zod de cada resposta replica à mão o que
a API devolve, então uma mudança de contrato na API pede a mudança correspondente aqui.

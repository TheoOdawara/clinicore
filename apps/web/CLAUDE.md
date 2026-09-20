# Clinicore `apps/web` — por feature e pegadinhas

Aditivo ao `CLAUDE.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este
arquivo apenas onde ela falar do mesmo assunto. A raiz tem a stack, os comandos, a visão geral da
arquitetura, as branches e o idioma.

## Por feature

**A dependência aponta para cima: `shared → features → routes`.**

- **Rota é fina:** declara o caminho, carrega dado no `loader` com
  `queryClient.ensureQueryData(<feature>QueryOptions)`, define `pendingComponent` e
  `errorComponent`, e compõe features. `routes/(app)/route.tsx` exige sessão.
- **Uma feature não importa outra.** Quem junta duas features é a rota.
- **`shared/` não tem regra de negócio.** Componente de domínio mora na feature.
- **Sem barrel file** (`index.ts` que só reexporta).
- Arquivo usado por uma única rota pode ficar colocado ao lado dela, em pasta com prefixo `-`
  (fora do roteamento).
- **`queryOptions` fica em `features/<feature>/api/`**, e o componente lê com `useSuspenseQuery`.
- **Toda resposta da API passa por `.parse()` de um schema Zod** em `features/<feature>/api/`, e o
  tipo sai de `z.infer`. O schema replica o DTO da API à mão.
- **`shared/http/` é a única instância do axios**, com `baseURL`, `withCredentials: true` e o
  interceptor que renova a sessão: um `401` dispara uma única chamada a `POST /auth/refresh`, as
  requisições concorrentes esperam essa chamada em vez de dispararem a sua, e o fracasso leva para
  `/login`. Nenhuma feature cria instância própria nem chama `axios` direto.
- **Formulário é TanStack Form** com o schema Zod nos validadores de blur e submit.
- **A UI base é shadcn/ui**, copiada para `shared/`, e o que o shadcn cobre não é reescrito à mão. O
  React Bits entra pela tabela de categorias da ADR 0003 — no `apps/web` não há fundo animado, efeito
  de cursor nem texto animado sobre dado clínico. Todo componente copiado respeita
  `prefers-reduced-motion`, nunca atrasa informação na tela, e perde os comentários na mesma edição.
- **O texto de erro que o usuário lê é pt-BR e mora no web**, traduzido a partir do `code` que a API
  devolve, dentro de `features/<feature>/api/`. `code` sem tradução cai numa mensagem genérica.
- **Testes:** lógica pura (schema, formatação, hook) e componente que decide algo (formulário, estado
  vazio ou de erro), com `features/*/api` substituído por `jest.fn`. Componente que só exibe não
  tem teste. Ficam em `__tests__/` ao lado do código testado, nos dois apps.
- **URL é inglês**, porque o nome da pasta de rota é o caminho: `/patients`, nunca `/pacientes`.

## Pegadinhas da stack

Verificadas em 2026-09-15 e reconfirmadas em 2026-09-20, já com o ESLint e o TypeScript 6.0 da
ADR 0004.

- **Nem o Vite nem o `@swc/jest` checam tipo.** Os dois apagam o tipo e seguem: um arquivo com erro
  de tipo roda, o teste passa e o build sai com código 0. Só o `tsc` pega, e por isso o
  `npm run typecheck` é um gate separado.
- **`import.meta.env` do Vite não existe no Jest.** No modo CommonJS, o arquivo que o lê derruba a
  suíte com `Must use import to load ES Module`; no modo ESM, carrega e o valor chega `undefined`.
- **O `shadcn add` e o `jsrepo` geram arquivo comentado**, e o repo só admite o marcador `ponytail:`. A
  limpeza é parte de adicionar o componente, não uma passada depois.
- **Parte dos componentes do React Bits ignora `prefers-reduced-motion`.** Conferir e corrigir no
  arquivo copiado; não dá para presumir que a fonte respeitou.

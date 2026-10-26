# Clinicore `apps/web` — por feature e pegadinhas

Aditivo ao `AGENTS.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este
arquivo apenas onde ela falar do mesmo assunto. A raiz tem a stack, os comandos, a visão geral da
arquitetura, as branches e o idioma.

## Por feature

**O `src/` hoje tem só `routes/`, `shared/env/` e `styles/`.** `features/` e `shared/http/` nascem com a
primeira feature, e as regras abaixo valem a partir dela.

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
  interceptor que renova a sessão: um `401` dispara uma única chamada a `POST /sessions/current/tokens`, as
  requisições concorrentes esperam essa chamada em vez de dispararem a sua, e o fracasso leva para
  `/login`. Nenhuma feature cria instância própria nem chama `axios` direto.
- **Formulário é TanStack Form** com o schema Zod nos validadores de blur e submit.
- **Animação e interatividade são Motion e React Bits.** O React Bits entra pela tabela de categorias da
  ADR 0003: no `apps/web` não há fundo animado, efeito de cursor nem texto animado sobre dado clínico, e
  a variante em Motion de cada componente é a preferida; componente que puxe GSAP, three.js ou ogl não
  entra. Todo componente copiado respeita `prefers-reduced-motion`, nunca atrasa informação na tela, e
  perde os comentários na mesma edição.
- **O texto de erro que o usuário lê é pt-BR e mora no web**, traduzido a partir do `code` que a API
  devolve, dentro de `features/<feature>/api/`. `code` sem tradução cai numa mensagem genérica.
- **Testes cobrem 100% de `src/`**, fora o `routeTree.gen.ts`, com o Vitest e o happy-dom. Lógica pura
  tem teste unitário; tela tem teste de integração que renderiza a rota de verdade, com Router, Query, a
  instância do axios e o interceptor, e o HTTP interceptado pelo MSW. O `vi.mock` fica para o que não é
  HTTP. Componente que só exibe é coberto pela rota que o usa. O e2e é o Playwright contra a API real.
  Ficam em `__tests__/` ao lado do código testado.
- **URL é inglês**, porque o nome da pasta de rota é o caminho: `/patients`, nunca `/pacientes`.

## Pegadinhas da stack

Verificadas em 2026-10-25, com o Bun 1.4.4, o Vite 8.3.5, o Vitest 5.0.5 e o TypeScript 7.0.2 da
ADR 0019.

- **Nem o Vite nem o Vitest checam tipo.** Os dois apagam o tipo e seguem: um arquivo com erro de tipo
  roda, o teste passa e o build sai com código 0. Só o `bun check` pega, e por isso ele é um gate
  separado.
- **O plugin do TanStack Router injeta código de HMR em toda rota**, e esse código nunca roda no teste,
  o que derruba a cobertura da rota abaixo de 100%. O `vite.config.ts` desliga isso no modo `test`, com
  `codeSplittingOptions.addHmr`.
- **O `bun fmt` lê o `.prettierignore` no lugar do `.gitignore`** quando o arquivo existe. Por isso ele
  repete o `coverage` e o `dist`, além do `routeTree.gen.ts`, que o plugin gera no próprio formato.
- **Nenhum script do `package.json` se chama `check` nem `fmt`.** O `bun check` e o `bun fmt` rodam o
  script de mesmo nome quando ele existe.
- **O `VITE_API_URL` entra no build, não no container.** O Dockerfile recebe por `--build-arg` e
  reprova o build sem ele; trocar a URL da API é um build novo.
- **O `jsrepo` copia o componente do React Bits com comentários.** A limpeza é parte de adicionar o
  componente, não uma passada depois.
- **Parte dos componentes do React Bits ignora `prefers-reduced-motion`.** Conferir e corrigir no
  arquivo copiado; não dá para presumir que a fonte respeitou.

---
name: web-feature
description: O passo a passo de uma feature nova ou de uma tela nova no apps/web do Clinicore, em Angular. Use ao criar ou mudar feature, tela, store ou chamada à API no apps/web.
---

# web-feature

A forma (pastas, fronteiras, nomes) está no `apps/web/AGENTS.md`.

## Feature nova

1. Criar `src/app/features/<feature>/` com `<feature>.routes.ts`, que exporta as rotas como `default`.
2. Ligar a feature no `app.routes.ts` por `loadChildren`, atrás do `auth-guard` quando a rota é
   autenticada.
3. Em `data/`:
   - `<entidade>.ts`: o schema Zod da resposta, com o nome do schema da API (`PatientResponse`), e o
     tipo por `z.infer`;
   - `<feature>-api.ts`: o `HttpClient` contra a URL do `app-config.ts`, sem estado;
   - `<feature>-store.ts`: o `signalStore` da feature, que chama a API, faz o `.parse()` e guarda o
     estado de carga e de erro;
   - `<feature>-errors.ts`: o mapa de `code` da API para texto pt-BR.
4. A store entra em `providers` da rota da feature, não em `root`.

## Tela nova

1. Criar `pages/<tela>/` com `<tela>.ts`, `.html`, `.scss` e `.spec.ts`.
2. A tela injeta a store com `inject()` e só lê os sinais dela e chama os métodos dela.
3. Pedaço de apresentação vai para `ui/<componente>/`, com `input()` e `output()`. Se mais de uma feature
   usa, vai para `shared/`.
4. Formulário é Signal Forms com o schema Zod por `validateStandardSchema`, e o erro de campo da API
   (`pointer`) volta para o campo.
5. O texto é pt-BR no template; data e número passam pelo `LOCALE_ID` pt-BR.

## Testes

- Store e API: Vitest com `HttpTestingController`, conferindo a URL, o corpo e o tratamento de um
  payload fora do schema.
- Tela: Testing Library Angular, pelo que a pessoa vê e clica.
- Fluxo: Playwright em `e2e/<feature>/`, contra a API local.

# `apps/web` — forma e regras

Aditivo ao `AGENTS.md` da raiz. Stack na ADR 0020, forma na 0026. É o sistema da clínica em
`app.clinicore.com.br`: SPA estático em Angular, instalável como PWA, só para usuário autenticado e
sempre online.

## Estrutura

```
apps/web/
├── public/                  manifest.webmanifest, ícones do PWA, favicon
├── e2e/<feature>/<fluxo>.spec.ts · e2e/support/     Playwright
└── src/
    ├── main.ts  index.html
    ├── styles/              styles.scss é manifesto: _tokens.scss, _theme.scss (mat.theme), _typography.scss
    └── app/
        ├── app.ts  app.html  app.scss  app.config.ts  app.routes.ts
        ├── shell/           o layout autenticado: barra, menu, conteúdo
        ├── core/            peça única do app, sem tela e sem regra de feature
        │   ├── config/      app-config.ts: único leitor do ambiente, validado por Zod
        │   ├── http/        api-error.ts (Problem Details), credentials-interceptor.ts, refresh-interceptor.ts
        │   ├── session/     session-store.ts (o usuário atual), auth-guard.ts
        │   └── locale/      LOCALE_ID, MAT_DATE_LOCALE e os textos do Material em pt-BR
        ├── shared/<componente>/    UI genérica de mais de uma feature, sem estado e sem HTTP
        └── features/<feature>/
            ├── <feature>.routes.ts     rotas lazy, com a store em providers
            ├── data/        <entidade>.ts (schema Zod e tipo), <feature>-api.ts, <feature>-store.ts, <feature>-errors.ts
            ├── pages/<tela>/           ligada a rota, lê a store
            └── ui/<componente>/        apresentação: só input() e output()
```

## Fronteiras

Conferidas pelo `eslint-plugin-boundaries`. Cada pasta importa só:

- `features/X`: `core`, `shared` e a própria X, nunca outra feature;
- `pages/`: `ui/` e `data/` da própria feature;
- `ui/`: `shared` e outro `ui/` da própria feature;
- `data/`: `core/http` e `core/config`;
- `shared` e `core`: só a si mesmos;
- `shell` e a raiz do `app/`: tudo.

## Padrões

- **Componente standalone, sem zone e com `OnPush`.** `inject()` no lugar do construtor; membro que só o
  template usa é `protected`; `input()`, `output()` e `model()` são `readonly`. Template e SCSS sempre
  em arquivo próprio.
- **Toda feature é lazy**, pelo `loadChildren` do `app.routes.ts`.
- **A store é NgRx SignalStore, uma por feature**, só da parte estável do `@ngrx/signals`. Estado local
  é `signal()` no componente.
- **Toda resposta da API passa por `.parse()` do Zod dentro do método da store.** O `code` de erro da API
  vira texto pt-BR no `<feature>-errors.ts`.
- **Formulário é Signal Forms com o schema Zod** por `validateStandardSchema`.
- **O `HttpClient` usa `withCredentials`, e o `refresh-interceptor` faz um único refresh no 401.**
- **Estilo é SCSS por componente sobre os tokens do tema do Material**; sem Tailwind e sem valor de cor
  ou espaço solto.
- **Animação é CSS com `animate.enter`/`animate.leave`, atrás de `prefers-reduced-motion`.**
- **Texto em pt-BR direto no template**; sem biblioteca de i18n.
- **O ambiente entra no build pelo `--define`** e é validado no boot pelo `app-config.ts`. Trocar a URL
  da API é um build novo.
- **O service worker não guarda resposta da API**: o web não tem dado offline.

## Nomes

Arquivo em kebab-case. Componente, serviço e diretiva sem sufixo (`patient-list.ts` exporta
`PatientList`); guard, interceptor e pipe com sufixo (`auth-guard.ts`); store e API com o papel no nome
(`PatientsStore`, `PatientsApi`). Seletor com o prefixo `app`. Teste ao lado, em `.spec.ts`. A URL é
inglês, kebab-case e no plural do recurso.

## Caminhos críticos

100% das linhas novas (ADR 0023): `core/http/`, `core/session/` e `core/config/`.

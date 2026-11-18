# 0020. Web em Angular

- Status: accepted, emendada pela 0026 na estrutura e nas convenções
- Date: 2026-11-17
- Substitui: 0019 na tabela do `apps/web` e 0018 no `apps/web`
- Emenda a: 0002 (o web deixa de ser Vite com React) e 0003 (o design system do web passa a ser o
  Angular Material)

## Context

O `apps/web` ainda é scaffold: nenhuma tela de agenda, paciente ou prontuário foi escrita, e a #78, a
primeira feature, não começou. É o último momento em que trocar o framework não reescreve nada.

O dono do produto decidiu levar o sistema da clínica para o Angular e revisou a stack dele peça por peça,
com cada versão conferida no registro do npm em 2026-11-17. O Angular 22 exige TypeScript `>=6.0 <6.1`,
então o TypeScript 7.0 da 0019 não vale para o web.

## Decision

**O `apps/web` é um SPA estático em Angular 22, instalável como PWA e servido pelo nginx.**

| Papel | Escolha | Versão em 2026-11-17 |
| --- | --- | --- |
| Pacotes | Bun instala e trava o `bun.lock` | 1.4.4 |
| Runtime das ferramentas | Node roda o `ng` | 26.13.1 |
| Framework | Angular, com o builder `@angular/build` | 22.2.6 · 22.2.7 |
| Tipos | TypeScript | 6.0.3 |
| Entrega | SPA estático, PWA pelo `@angular/service-worker` sem `dataGroups` da API, `dist/` servido pelo nginx | — |
| HTTP | `HttpClient` com interceptor funcional: `withCredentials` e um único refresh por vez no 401 | — |
| Estado de feature | NgRx SignalStore, uma store por feature, só a parte estável (`@ngrx/signals`, sem `@ngrx/signals/resource` e sem o ngrx-toolkit) | 22.0.1 |
| Estado local | `signal()` no componente | — |
| Formulário | Signal Forms com schema Zod por `validateStandardSchema` | — |
| Resposta da API | `.parse()` do Zod em toda resposta, dentro do método da store | 4.6.7 |
| Componentes | Angular Material, com os tokens da #162 alimentando o `mat.theme` | 22.2.7 |
| Estilo | SCSS por componente e o tema do Material em SCSS, sem Tailwind | — |
| Animação | CSS nativo com `animate.enter` e `animate.leave`, atrás de `prefers-reduced-motion`; sem `@angular/animations` | — |
| Fonte | `@fontsource-variable/atkinson-hyperlegible-next` | 5.3.0 |
| Lint e formato | angular-eslint com typescript-eslint type-aware · Prettier | 22.7.0 · 3.10.1 |
| Testes | Vitest pelo builder `@angular/build:unit-test`, com jsdom, Testing Library Angular e `HttpTestingController` | 5.1.0 · 31.0.1 · 19.6.0 |
| E2E | Playwright contra a API real | 1.65.1 |
| Idioma | Texto em pt-BR direto no template; `LOCALE_ID` pt-BR com `registerLocaleData`, `MAT_DATE_LOCALE` pt-BR, os textos internos do Material em pt-BR, e o `code` de erro da API traduzido na feature | — |

A arquitetura de cada feature segue o guia oficial do Angular. A forma fica no `apps/web/AGENTS.md`, o
passo a passo numa skill do projeto, e as fronteiras são conferidas pelo lint.

## Consequences

- **O web tem dois runtimes**, como o site tinha: o Bun instala e o Node roda o `ng`. O job do CI usa o
  `setup-bun` e o `setup-node`.
- **O builder de teste do Angular não tem modo "só os testes ligados ao que mudou".** Ele aceita globs
  em `include`, então o CI escolhe os testes por pasta de feature (0024).
- **O oxlint sai do web.** Ele não lê template Angular; o angular-eslint lê.
- **Se o Material incomodar, o plano B já está escolhido:** Angular Aria com o CDK, componentes próprios e
  Tailwind. A troca é uma ADR nova.
- **Motion só entra se uma tela desenhada pedir** uma animação que o CSS não faz.
- **A #78 precisa ter o corpo reescrito** para o Angular antes de começar.

## Alternatives considered

- **Manter o React com Vite e TanStack Router (0002 e 0019)** · rejeitada pelo dono do produto.
- **TanStack Query para Angular** · rejeitada: o adaptador do Angular era experimental, e o SignalStore
  estável resolve o estado de servidor por feature.
- **`@ngrx/signals/resource` e ngrx-toolkit** · rejeitados por não serem estáveis.
- **Tailwind no web** · rejeitado: o Material já entrega o sistema visual, e o tema dele é SCSS.
- **`@angular/animations`** · rejeitado: o Angular 22 anima entrada e saída com `animate.enter` e
  `animate.leave` sobre CSS nativo.
- **oxlint** · rejeitado: não lê template Angular.
- **Karma ou Jest** · rejeitados: o builder oficial do Angular roda o Vitest.
- **Biblioteca de i18n** · rejeitada: o sistema é só pt-BR.

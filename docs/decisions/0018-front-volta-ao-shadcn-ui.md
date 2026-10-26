# 0018. Front volta ao Tailwind e ao shadcn/ui

- Status: accepted
- Date: 2026-10-25
- Substitui: 0016 inteira; a 0003 volta a valer inteira

## Context

A ADR 0016 tirou o Tailwind e o shadcn/ui do `apps/web` e do `apps/site` e trocou os dois por CSS puro,
com CSS Modules. A troca era a issue #77, que não começou: os dois apps seguem com o Tailwind 4.3
instalado e nenhuma tela.

## Decision

**O `apps/web` e o `apps/site` continuam no Tailwind 4.3, e o shadcn/ui volta a ser o design system dos
dois, como a ADR 0003 decidiu.** A #77 é fechada sem ser feita, e o shadcn é configurado na #78, junto
com o primeiro componente copiado.

## Consequences

- **A regra 8 da spec `003-autenticacao-web.md` volta a vir do Radix**: rótulo associado,
  `aria-describedby` e foco visível chegam com o componente copiado, e o `role="alert"` do erro é
  conferido nele.
- **As classes utilitárias voltam ao JSX**, e o Radix e o CVA voltam a ser dependências de runtime, por
  componente usado.
- **Nada do que a 0016 previa entra**: nem as camadas do `globals.css`, nem o `.module.css` por
  componente, nem o `@eslint/css`, nem o Base UI como primitiva.

## Alternatives considered

- **Seguir a ADR 0016, com CSS Modules** · rejeitada pelo dono do produto.
- **Tailwind sem o shadcn** · rejeitada pelo dono do produto: a regra 8 seria escrita à mão.

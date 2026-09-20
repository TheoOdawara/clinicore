# 0004. Lint do front em ESLint, com TypeScript 6

- Status: accepted
- Date: 2026-09-20
- Emenda a: 0002 (ferramental de lint e versão do TypeScript do front)

## Context

O dono do produto pediu que o lint reprove **uso** de símbolo obsoleto — função chamada, método
chamado, propriedade lida —, não só o `import` dele, e nos três apps.

O `apps/api` já cumpria: o preset `strictTypeChecked` do `typescript-eslint` liga
`@typescript-eslint/no-deprecated` como `error`, e ela aponta a linha da chamada. Medido:

```
5:35  error  `calculate` is deprecated. use compute instead   @typescript-eslint/no-deprecated
6:29  error  `legacyHelper` is deprecated. use modernHelper   @typescript-eslint/no-deprecated
```

O `apps/web` não cumpria, e não é questão de ligar uma regra. O Biome 2.5 tem exatamente duas regras
com "deprecated" no nome — `suspicious/noDeprecatedImports` e `style/noDeprecatedMediaType` —, a
primeira não está no `recommended`, e mesmo declarada como `error` ela sinaliza o `import`, nunca o
uso. A causa é estrutural: reconhecer que `service.calculate` resolve num método anotado exige lint
**type-aware**, que o Biome não faz em versão nenhuma.

A ADR 0002 decidiu "uma stack só no front" — mesmo lint, mesmo formatador, mesma versão de
TypeScript no `apps/web` e no `apps/site`. Essa decisão continua valendo. O que muda é qual
ferramenta ocupa o lugar.

O momento é o mais barato que vai existir: o `apps/web` tem sete arquivos de código, um deles
gerado, e o `apps/site` ainda não tem nenhum.

## Decision

**O `apps/web` troca o Biome por ESLint e Prettier**, e o `apps/site` nasce assim na #76.

| | versão | papel |
| --- | --- | --- |
| `eslint` | 9.39.5 | motor |
| `typescript-eslint` | 8.70.0 | parser e presets `strictTypeChecked` e `stylisticTypeChecked`, com `projectService` |
| `eslint-plugin-jsx-a11y` | 6.10.2 | repõe o grupo `a11y` do `recommended` do Biome |
| `eslint-plugin-react-hooks` | 7.1.1 | repõe `useExhaustiveDependencies` e `useHookAtTopLevel` |
| `eslint-plugin-react-refresh` | 0.5.7 | preset `vite` |
| `prettier` e `eslint-config-prettier` | 3.9.8 e 10.1.8 | formato, nas mesmas versões da API |

**O `apps/web` cai de TypeScript 7.0 para 6.0.3.** Não é escolha: o `typescript-eslint@8.70.0`, que é
o `latest`, declara peer `typescript: ">=4.8.4 <6.1.0"`. Lint type-aware no front custa a versão do
compilador, o mesmo preço que a ADR 0001 já tinha pago na API. O `tsc --noEmit` do web passou sem
nenhum ajuste no `tsconfig.json`: nada ali usa o que o 6.0 removeu.

**O front fica em ESLint 9, e a API segue em ESLint 10.** O `eslint-plugin-jsx-a11y@6.10.2` declara
peer `eslint: ^3 … ^9`. Sem ele o web perderia as regras de acessibilidade que o `recommended` do
Biome entrega hoje como erro, e acessibilidade não se simplifica fora. Os apps são independentes por
desenho — lockfile próprio, zero import cruzado —, então duas majors de ESLint não acoplam nada. O
front sobe para a 10 quando o plugin declarar suporte.

**O script de gate continua se chamando `check`**, agora rodando
`eslint . --max-warnings 0 && prettier --check .`. Manter o nome deixa o `ci.yml` e a tabela de
comandos do `CLAUDE.md` intactos.

**`react-refresh/only-export-components` fica desligada em `src/routes/`.** A rota do TanStack Router
exporta `Route` e mantém o componente local, então a regra reprova toda rota. A opção
`allowExportNames` não cobre o caso: ela permite export extra ao lado de um componente exportado, e
aqui não há componente exportado nenhum.

## Consequences

- **`no-deprecated` passa a valer nos três apps**, no ponto de uso. É o que foi pedido.
- **Os sete arquivos do web foram reformatados** — o Biome usa tab, o Prettier dos três apps usa
  espaço. Ruído de diff pago uma vez.
- **Some a ordenação automática de import**, que vinha do `assist.organizeImports` do Biome. O ESLint
  não repõe isso e a API também não tem. Ninguém pediu para repor.
- **O gate de análise estática do front fica mais lento**, porque type-aware constrói o programa do
  TypeScript. Em sete arquivos é imperceptível; é custo que cresce com o app.
- **O front perde o compilador nativo do TypeScript 7.** O ganho era velocidade de `tsc`, não recurso
  de linguagem, e nenhum arquivo do web dependia de algo do 7.0.
- **A #76 nasce com este ferramental**, não com Biome.

## Alternatives considered

- **Biome e ESLint lado a lado no web**, o Biome como formatador e dono do resto do lint e o ESLint
  rodando só `no-deprecated` · rejeitada pelo dono do produto, que pediu a troca inteira. Custava
  duas ferramentas, dois arquivos de config e dois passos no gate, e ainda assim obrigava o
  TypeScript 6, que é o custo real.
- **ESLint 10 no web, instalando o `jsx-a11y` com o peer insatisfeito** · rejeitada: alinharia a
  major com a API, mas o ESLint 10 removeu API legada e o plugin não declara suporte. Quebraria no
  meio da #77 ou da #78, que são justamente as entregas de tela.
- **ESLint 10 no web sem o `jsx-a11y`** · rejeitada: o web perderia lint de acessibilidade que tem
  hoje, às vésperas das entregas de tela.
- **Ficar no Biome e aceitar só `noDeprecatedImports`** · rejeitada: entrega o import e não o uso,
  que é exatamente o que foi pedido. Medido: um método obsoleto chamado passa verde.
- **Esperar o Biome ganhar análise type-aware** · rejeitada: o Biome tem regras type-aware em
  `nursery` com inferência própria, nenhuma delas de código obsoleto, e não há data.

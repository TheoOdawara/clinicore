# 0026. Estrutura e convenções dos apps

- Status: accepted
- Date: 2026-11-17
- Emenda a: 0010 (a fronteira entre features da API e as peças transversais que vêm), 0020 (a
  estrutura do web), 0021 (a estrutura do site) e 0022 (os módulos do Android e os targets do iOS)

## Context

As ADRs 0020 a 0025 decidiram a stack de cada app. Nenhuma decidiu a forma do código: a árvore de
pastas, os nomes, os padrões e o que separa uma parte da outra. Sem isso, cada sessão de agente escolhe
uma forma, e a próxima escolhe outra.

O dono do produto decidiu a forma de cada app, um por vez, com o guia oficial de cada plataforma e o que
o mercado usa. A regra escrita não basta: a fronteira entre as partes de cada app é conferida por
ferramenta, para o agente não sair dela.

## Decision

**Em todo app, o código se organiza por feature, e a fronteira entre as partes é conferida por
ferramenta.** A árvore completa e as regras de cada app ficam no `AGENTS.md` dele; esta ADR registra a
decisão e o que a confere.

| App | Forma | Quem confere a fronteira |
| --- | --- | --- |
| `apps/api` | Um crate por processo (`core`, `app`, `worker`); no `app`, uma pasta por recurso com `mod.rs`, `handlers.rs`, `requests.rs`, `responses.rs`, `service.rs`, `queries.rs` e `error.rs` conforme o conteúdo | A visibilidade do Rust: `queries` é sempre privado, e a feature expõe só `service`, `error`, `handlers` e `extractors`. O resto, um teste em `crates/app/tests/api/architecture.rs` |
| `apps/web` | `app/` com `shell/`, `core/`, `shared/` e `features/<feature>/` com `data/`, `pages/` e `ui/` | `eslint-plugin-boundaries` |
| `apps/site` | `src/routes/` e `src/lib/` com `sections/`, `components/`, `assets/`, `content/` e `styles/` | `eslint-plugin-boundaries` |
| `apps/android` | Módulos `app`, `core/*` (`model`, `common`, `security`, `network`, `database`, `datastore`, `data`, `sync`, `designsystem`, `ui`, `testing`) e `feature/<feature>/{api,impl}` | O grafo de módulos do Gradle e testes do Konsist |
| `apps/ios` | Target do app no `Clinicore.xcodeproj` e um pacote local com os targets `Models`, `Networking`, `Persistence`, `Security`, `Repositories`, `Sync`, `DesignSystem`, `TestSupport` e `<Feature>Feature` | As dependências do `Package.swift` e regras próprias do SwiftLint |

Regras que valem para todos:

1. **Uma feature nunca importa outra.** No web e no site, quem junta as features é a rota; no Android,
   o `app` e o `api` de cada feature; no iOS, o target do app. Na API, uma feature chama o `service`
   de outra, nunca o `queries`.
2. **A tela nunca fala com a rede ou com o banco direto.** Ela passa pela store do web, pelo
   repository do Android e do iOS.
3. **Nomes seguem a linguagem.** Arquivo em kebab-case no web e no site, em snake_case no Rust, e em
   PascalCase com o nome do tipo no Kotlin e no Swift, que é a regra dessas duas linguagens.
4. **O nome de um schema da API é o nome do tipo em todo cliente**: `<Operação>Request` e
   `<Recurso>Response`.
5. **Cada app tem um único leitor do ambiente**, validado no boot.

Na API, os IDs ganham tipo próprio (`UserId`, `ClinicId`, `PatientId`) e o e-mail normalizado vira
`Email`. As peças transversais que vêm (`tenancy/`, `authorization/`, `audit/`, `idempotency/`) moram em
`src/`, como o `credentials/`.

Os caminhos críticos de cada app, que têm 100% das linhas novas cobertas pela 0023, ficam no `AGENTS.md`
dele.

## Consequences

- **Os módulos do Android passam de três `core` para onze**, no desenho do Now in Android. Os plugins de
  convenção do `build-logic` absorvem a configuração repetida.
- **Os targets do iOS passam de três para oito.** O `Package.swift` fica mais longo, e em troca o
  compilador esconde das telas o SwiftData, o Keychain e o `URLSession`.
- **Kotlin e Swift saem da regra de nome em kebab-case** do contrato global, por exigência das duas
  linguagens.
- **A API precisa de um refactor** para os IDs tipados, o `queries` privado e o teste de arquitetura.
  Ele vira uma issue.
- **O web e o site ainda são o scaffold em React e em Next.** O `AGENTS.md` de cada um já descreve a
  forma decidida, e a troca de framework é uma issue de cada app.

## Alternatives considered

- **Regra só escrita, sem ferramenta** · rejeitada pelo dono do produto: o agente sai da forma que não
  é conferida.
- **Hexagonal na API** · rejeitada: o teste já roda contra o Postgres real, e as traits e os mocks
  custariam código sem trocar nada que exista.
- **Pasta por tipo de arquivo** (`components/`, `services/`, `controllers/`) · rejeitada: uma mudança
  numa feature fica espalhada por várias pastas.
- **Feature plana no web, sem `data`, `pages` e `ui`** · rejeitada: o lint não conseguiria impedir um
  componente de apresentação de chamar a API.
- **Um pacote SwiftPM por feature no iOS** · rejeitado: só compensa com equipes separadas.
- **Kebab-case em todo arquivo, inclusive Kotlin e Swift** · rejeitado: o detekt, o SwiftLint e as IDEs
  das duas linguagens esperam o nome do tipo.

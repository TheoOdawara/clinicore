# Clinicore — contrato do repositório

Aditivo ao contrato global; em conflito, este arquivo vence. O `docs/README.md` é o índice da
documentação.

## Stack

Cinco apps independentes em `apps/`, cada um com manifesto, lockfile, gates e `AGENTS.md` próprios.
Nenhum código é compartilhado: a fronteira é o contrato HTTP.

| App | Stack | Domínio |
| --- | --- | --- |
| `apps/api` | Rust · axum · sqlx · PostgreSQL · Redis | `api.clinicore.com.br` |
| `apps/web` | Angular · Angular Material · NgRx SignalStore · Bun instala, Node roda o `ng` | `app.clinicore.com.br` |
| `apps/site` | SvelteKit estático · Bun instala, Node roda as ferramentas | `clinicore.com.br` |
| `apps/android` | Kotlin · Jetpack Compose · Room com SQLCipher · Hilt | loja |
| `apps/ios` | Swift · SwiftUI · SwiftData · SwiftPM | loja |

As versões estão nas ADRs, e depois nos manifestos. Cada app nasce no scaffold dele, já na forma do seu
`AGENTS.md`.

## Linguagens e scripts

Código de aplicação em Rust, TypeScript, Kotlin e Swift, um por app. Script de um app fica nele e é
escrito na linguagem dele; ferramenta fora de qualquer app é Python em `tools/` (ADR 0025). Nenhum
script `.sh`.

## Arquitetura

```
apps/              api · web · site · android · ios
docs/              product · system · decisions · work
.agents/skills/    skills do projeto (.claude/skills/ aponta para elas)
.github/workflows/ ci-<app>.yml por app, com o scaffold dele · ci-security · ci-repo · ci-audit
```

- **O browser fala com a API em origem cruzada**, no mesmo domínio registrável, o que mantém o cookie
  em `SameSite=Lax`; o CORS libera `ALLOWED_ORIGINS` com credenciais. O app nativo usa token no header
  (`Clinicore-Client: mobile`).
- **O offline é só dos apps nativos** (ADR 0007 nos itens 3 a 9, e 0022).
- **Deploy pelo Dockerfile de cada app no Coolify** (ADR 0014); o único compose é o de dev da API.
- **A forma de cada app está no `AGENTS.md` dele** (ADR 0026): árvore, fronteiras, nomes e caminhos
  críticos.

## Convenções

- **Uma feature nunca importa outra**, e a tela nunca fala com a rede ou o banco direto. A fronteira é
  conferida por ferramenta em cada app.
- **O JSON da API é camelCase**, e o nome do schema (`<Operação>Request`, `<Recurso>Response`) é o nome
  do tipo em todo cliente.
- **Cobertura:** 90% das linhas novas de cada PR, e 100% nos caminhos críticos (ADR 0023).
- **O repo é público** (ADR 0024): nenhum segredo, dado real, nome da clínica piloto ou endereço da VPS
  entra nele.
- **Nunca rodar** `infisical secrets`, `infisical export`, `env`, `printenv` nem
  `docker compose config`: imprimem valores.

## Branches

`main` é produção e `staging` é homolog, protegidas só por convenção: nunca commitar nelas. Cada entrega
nasce em `feature/<issue>-<assunto>` a partir da `staging` e volta por PR; correção urgente é
`hotfix/<issue>-<assunto>` a partir da `main`. Release é PR da `staging` para a `main`.

## Idioma

Documentação em pt-BR. Código, identificadores, pastas, arquivos, branches e commits em inglês. A
mensagem de commit é só o título; o porquê vai na descrição do PR.

## Skills do projeto

- `gates` — os gates de cada app, antes de dar algo por pronto.
- `testing` — ao escrever ou revisar teste, e a cobertura.
- `secrets` — ao precisar de variável de ambiente em desenvolvimento.
- `web-feature` — ao criar ou mudar feature no `apps/web`.
- `site-section` — ao criar ou mudar seção do `apps/site`.
- `android-feature` — ao criar ou mudar feature no `apps/android`.
- `ios-feature` — ao criar ou mudar feature no `apps/ios`.
- `design` — ao desenhar marca ou tela.
- `backlog` — ao criar issue, abrir PR ou mover item no Project.
- `dependency-update` — no fechamento da sprint e num alerta do Dependabot.

# 0024. CI por mudança, num repositório público

- Status: accepted
- Date: 2026-11-17
- Emenda a: 0019 (o gate de CVE do iOS e a forma de atualizar dependência)

## Context

O repositório é público no GitHub, sem arquivo de licença, com a varredura de segredos desligada, e o CI
roda num workflow só, com um job por app, inteiro, em todo PR. Com os apps Android e iOS entrando, um PR
que só mexe no site subiria emulador e Mac.

Uma varredura do histórico com o gitleaks em 2026-11-17 achou um único valor, um `JWT_SECRET` de teste do
CI no commit `7486536`.

## Decision

**Repositório público**

1. **Licença proprietária**, com todos os direitos reservados, num `LICENSE` na raiz. O código fica
   visível e nenhum uso é concedido.
2. **A varredura de segredos do GitHub e o bloqueio de push com segredo ficam ligados.**
3. **O gitleaks roda em todo PR**, com o binário numa versão travada.
4. **Os workflows usam só `pull_request`, nunca `pull_request_target`**, com `permissions: contents:
   read`. O CI de PR só recebe valores descartáveis de teste. Os segredos de produção ficam no Coolify, e
   os de publicação dos apps em environments protegidos do GitHub.
5. **A configuração de ambiente fica fora do git em cada app**, com só o exemplo commitado. O material de
   assinatura dos apps, a keystore do Android e os certificados da Apple, fica só nos segredos.
6. **Nenhum dado real entra no repositório**: fixture e seed com dado inventado, sem nome de paciente ou
   da clínica piloto, sem endereço interno da VPS.
7. **Do Dependabot, só os alertas ficam ligados.** As correções automáticas ficam desligadas e não há
   `.github/dependabot.yml`, então ele não abre PR. As dependências são atualizadas num PR só, no
   fechamento de cada sprint; um alerta grave vira `hotfix/`.

**CI**

1. **Um workflow por app**, com o filtro nativo de caminho do GitHub: `ci-api.yml`, `ci-web.yml`,
   `ci-site.yml`, `ci-android.yml` e `ci-ios.yml`. Os gatilhos são PR para a `staging` e para a `main`.
   Um PR só de documentação não roda nenhum app.
2. **Dentro de cada workflow, roda só o que a mudança afeta e o que depende dela.** Um job `changes`
   lê os arquivos alterados com o `dorny/paths-filter` v4.0.3, travado pelo hash do commit, e libera os
   jobs afetados.

   | App | O que dispara o quê |
   | --- | --- |
   | `apps/api` | `core` → os três crates · `app` → `app` · `worker` → `worker` · `migrations/` ou `.sqlx/` → `app`, `worker` e o `sqlx prepare --check` · `Cargo.lock` ou `deny.toml` → `cargo deny` e tudo |
   | `apps/web` | uma feature → os testes dela, pelo `include` do builder, e o e2e marcado com ela · `shared/`, configuração ou `package.json` → todos · lint e formato nos arquivos alterados · `ng build` sempre |
   | `apps/site` | `vitest --changed` contra a `staging` · lint e formato nos arquivos alterados · `svelte-check` e build sempre |
   | `apps/android` | uma feature → ela e o `app` · `core/model` ou `core/data` → o que depende deles · `build-logic` ou o catálogo de versões → tudo · emulador só quando o `core/data` muda · Maestro com os fluxos da feature · cache de build do Gradle |
   | `apps/ios` | o mesmo desenho, com `swift test --filter` nos targets afetados e `-only-testing:` no XCUITest · formato e lint nos arquivos alterados · build do app sempre |

3. **O push na `staging` e o PR para a `main` rodam tudo.** É a rede de segurança para um mapa de
   dependências desatualizado no YAML.
4. **Tipos, build e formato rodam o app inteiro** sempre que ele é afetado; a auditoria de CVE roda
   quando o lockfile muda.
5. **Máquinas:** `ubuntu-latest` para api, web, site e android, com a aceleração de emulador do runner
   Linux; `xcode-27`, com o Xcode travado no 27.2, para o iOS.
6. **Toda action de terceiro fica numa versão exata**, e o cache de dependência de cada app usa o
   lockfile dele como chave.

## Consequences

- **O mapa de dependências entre unidades é escrito à mão no YAML.** O `AffectedModuleDetector`, que
  calculava isso no Gradle, foi arquivado, e nenhuma ferramenta mantida faz o mesmo nos cinco apps.
- **A imagem `xcode-27` ainda é preview** no GitHub e pode ter instabilidade ou fila. Quando sair do
  preview, o workflow do iOS troca para o rótulo estável.
- **Como o repositório é público, as máquinas do CI não custam nada**, inclusive as de macOS.
- **O iOS não tem gate de CVE no CI.** O osv-scanner não lê o `Package.resolved`, e o alerta do
  Dependabot aparece no GitHub, não no build.
- **O `JWT_SECRET` do histórico precisa ser confirmado como valor de teste.** Se ele tiver sido usado em
  algum ambiente real, é trocado.

## Alternatives considered

- **Repositório privado** · rejeitado: o minuto de macOS do CI passaria a custar dez vezes o de Linux.
- **Licença MIT, Apache 2.0 ou AGPL 3.0** · rejeitadas: o produto é vendido. A FSL 1.1 foi considerada e
  ficou de fora em favor da proprietária.
- **`tj-actions/changed-files`** · rejeitado: foi comprometido num ataque de cadeia de suprimento em
  2025.
- **Um workflow só com o app inteiro em todo PR** · rejeitado: sobe emulador e Mac em PR que não mexe
  neles.
- **PRs automáticos do Dependabot** · rejeitados pelo dono do produto.

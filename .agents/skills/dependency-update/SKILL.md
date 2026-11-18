---
name: dependency-update
description: Como as dependências do Clinicore são atualizadas, sem PR automático do Dependabot. Use no fechamento de cada sprint e quando chegar um alerta do Dependabot.
---

# dependency-update

- **O Dependabot só alerta** (ADR 0024). As correções automáticas ficam desligadas e não há
  `.github/dependabot.yml`, então ele não abre PR.
- **No fechamento de cada sprint, um PR só atualiza as dependências de todos os apps**, cada uma para a
  última versão estável, travada na versão exata. A versão é conferida no registro do pacote no dia,
  nunca de memória.
- **Cada app atualiza pelo próprio gerenciador** e commita o lockfile: `Cargo.lock`, `bun.lock`, os
  `gradle.lockfile` com o `verification-metadata.xml`, e o `Package.resolved`.
- **Todos os gates de todos os apps rodam no PR de atualização** (skill `gates`).
- **Um alerta grave fora do fechamento da sprint vira `hotfix/<issue>-<assunto>`** a partir da `main`.
- **Uma advisory que não alcança o código fica ignorada com o motivo** no `deny.toml` da API, no campo
  `reason`, e sai quando a dependência corrigir.

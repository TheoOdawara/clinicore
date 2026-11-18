---
name: backlog
description: Como o backlog do Clinicore funciona no GitHub Project, com labels, issues gêmeas e status marcado à mão. Use ao criar issue, abrir ou mesclar PR, e ao escolher a próxima issue.
---

# backlog

- **O GitHub Project `Clinicore`**, em `https://github.com/users/TheoOdawara/projects/3`, **tem só a
  sprint atual**, no campo `Sprint`, com os pontos no campo `Estimate`. O roadmap mora no
  `docs/work/roadmap/README.md` e nos milestones. Uma issue entra no Project quando uma sprint a
  planeja, e o épico sem issue ganha uma nesse momento.
- **A branch padrão do GitHub é a `main`**, então a palavra de fechamento só fecha a issue quando o
  release chega nela. O PR para a `staging` não liga a issue nem move o item.
- **`In Review` e `Staging` são marcados à mão**, com `gh project item-edit`: `In Review` ao abrir o PR,
  `Staging` no merge na `staging`. O `Done` vem sozinho quando a issue fecha.
- **A ordem de entrega é API e banco, depois web e PWA, depois Android e iOS.** A próxima issue é a
  primeira aberta nessa ordem, respeitando o `Depende de` de cada uma.
- **Toda issue de tela ou de comportamento do `apps/web` tem duas gêmeas**, uma no `apps/android` e uma no
  `apps/ios`, criadas junto com ela: é o que sustenta a paridade da ADR 0022.
- **Toda issue leva a label do tipo do commit** (`feat`, `fix`, `refactor`, `docs`, `chore`, `ci`,
  `test`, `build`) **e a `area:<app>`** do que ela toca: `area:api`, `area:web`, `area:site`,
  `area:android`, `area:ios` ou `area:ci`.
- **A descrição do PR segue o `.github/pull_request_template.md`**: o porquê, o que mudou e como foi
  testado. O commit é só o título.

# 0007. App nativo em Flutter, com offline completo

- Status: accepted, emendada em 2026-09-21 pela spec `004-sessao-por-token-api.md` nos itens 7 e 8
- Date: 2026-09-21
- Emenda a: 0002 (o web deixa de ser o único front do sistema e não guarda dado clínico offline),
  `docs/requirements/` (a plataforma, hoje `NFR-FLEX-01`, e o offline, hoje `NFR-REL-02`, `NFR-REL-03` e
  `NFR-SEC-04`) e `CLAUDE.md` da raiz (Stack)

## Context

O documento de requisitos dizia "web responsiva, sem app nativo no MVP", e a ADR 0002 entregava o sistema
como PWA instalável. O dono do produto reviu isso: o dentista precisa ver a agenda e o paciente, e
continuar registrando o atendimento, sem internet — no consultório com sinal ruim, em atendimento fora da
clínica, numa queda do provedor.

O PWA não sustenta esse uso no iOS, que é onde está boa parte dos dentistas:

- **O Safari apaga o storage do site** — IndexedDB, cache e registro do service worker — após 7 dias de
  uso do Safari sem interação com o site. O app instalado na tela inicial tem contador próprio, mas o
  despejo por pressão de disco continua valendo (WebKit, "Updates to Storage Policy", 2023).
- **Background Sync não existe em nenhuma versão do Safari.** Uma escrita feita offline só sobe com o app
  aberto na tela.
- **Não há Keychain nem SQLite cifrado** no browser. Dado de saúde offline ficaria em IndexedDB, cifrado
  por uma chave que o próprio JavaScript precisa ler.

Na mesma conversa ficaram decididas as regras que o offline carrega: quem edita o prontuário, como se
resolve conflito, quanto tempo o dado fica no aparelho.

Hoje a API tem só `auth` e `health`, e o `apps/web` é scaffold. Nenhuma tela de agenda, paciente ou
prontuário existe em nenhum dos dois fronts: é o último momento em que decidir o segundo front não
reescreve nada.

## Decision

**Nasce o `apps/mobile`, app nativo iOS e Android em Flutter**, com paridade de telas com o `apps/web` e
entregue junto no go-live da clínica piloto. O web continua existindo, exige conexão e não guarda dado
clínico offline; o offline é exclusivo do app.

| Papel | Escolha | Versão conferida em 2026-09-21 |
| --- | --- | --- |
| SDK | Flutter, com o Dart que o acompanha | 3.47 |
| Cliente HTTP gerado do OpenAPI | `swagger_parser` (Retrofit sobre dio) | 1.44.3 |
| Banco local | `drift` sobre `sqlite3`, cifrado com SQLite3MultipleCiphers (`source: sqlite3mc`) | 2.35.0 · 3.6.0 |
| Token e chave do banco | `flutter_secure_storage` — Keychain no iOS, Keystore no Android | 11.2.0 |

Decisões de detalhe dentro desta:

1. **Gates do `apps/mobile`:** `flutter analyze`, `dart format --set-exit-if-changed .` e `flutter test`,
   num quarto job do `.github/workflows/ci.yml`. Pacotes por pub, com `pubspec.lock` commitado.
2. **O cliente HTTP é gerado do OpenAPI** que a API já publica pelo `@nestjs/swagger`. Gerar do contrato
   não é importar código: a fronteira continua sendo o HTTP. O `swagger_parser` foi escolhido sobre o
   `dart-dio` do `openapi-generator` 7.25.0 por ser Dart puro, sem Java no ferramental. O web continua
   replicando o DTO à mão em Zod. O modelo gerado cobre só o que trafega; banco local e fila têm modelo
   próprio.
3. **Offline completo, leitura e escrita.** O app funciona igual sem conexão. Toda escrita vai para uma
   fila local, em ordem, e sobe quando a conexão volta, sem ação da pessoa.
4. **Conflito se resolve por versão e pendência.** Todo registro que o app pode alterar carrega uma
   versão. Uma escrita da fila feita sobre versão velha — ou que ocupa um horário já ocupado — é recusada
   pela API, e o item vai para "Pendências" no app, para a pessoa refazer. Nada é sobrescrito em
   silêncio. Por isso a API passa a exigir, em toda rota de escrita que o app chama: coluna de versão no
   registro, identificador UUID gerado no cliente para registro criado offline, e chave de idempotência,
   porque a fila reenvia.
5. **Prontuário é editado só pelo dentista ou por assistente que ele designar**, e o dentista é
   notificado quando outra pessoa altera. É regra de negócio que reduz o conflito na origem; o
   mecanismo do item 4 continua valendo para o caso que sobra.
6. **O dado no aparelho vem em duas camadas.** A ocupação da agenda — horário livre ou ocupado, sem
   paciente — cobre um período longo, para responder "tem horário daqui a dois meses". O dado clínico
   cobre só os pacientes de um período curto. Os dois períodos são números da spec da agenda.
7. **Dado de saúde no aparelho** fica no banco cifrado, com a chave no Keychain/Keystore. **O app não
   tem trava própria**: a biometria é forma de login (passkey), não bloqueio de tela, e o dado offline
   fica protegido pela criptografia do banco e pelo bloqueio do aparelho, quando a pessoa configurou um.
   Quem deixa o aparelho sem bloqueio assume esse risco. É apagado **72 horas após a última sincronização**, no
   logout e quando a API recusa o refresh da sessão. A fila pendente não é apagada: sobe no próximo login.
8. **A sessão do app é token no header**, não cookie: o app não é o mesmo site da API, então o
   `SameSite=Lax` da spec `003-autenticacao-api.md` não se aplica a ele. Access token curto, refresh
   opaco guardado no Keychain/Keystore, os dois na mesma tabela `session` que o web usa. A sessão do
   app dura 7 dias de inatividade; a do web continua em 24 horas, porque o computador da recepção é
   compartilhado. O contrato está em `docs/specs/autenticacao/004-sessao-por-token-api.md`.
9. **Leitura de prontuário offline é auditada.** O app registra o acesso localmente e o envia pela mesma
   fila, para que a trilha de acesso ao prontuário do §5 dos requisitos não tenha buraco.

Fica fora desta ADR, cada um com o seu dono:

- Os dois períodos do item 6 — spec da agenda, depois de analisar o uso real.
- O canal da notificação do item 5 — spec do prontuário.
- As rotas de sincronização e o formato da pendência — spec do motor de sync.
- As rotas e o ciclo do token do item 8 — `docs/specs/autenticacao/004-sessao-por-token-api.md`.

Ordem do que vem depois: spec de autenticação do app, spec do motor de sync, scaffold do `apps/mobile`.

## Consequences

Fica mais fácil:

- O dentista trabalha sem internet, com o dado cifrado pelo sistema operacional e não pelo browser.
- A escrita offline sobe sozinha, sem depender de Background Sync.
- Conflito nunca apaga trabalho de ninguém: ou passa, ou vira pendência visível.
- O cliente HTTP do app acompanha a API por geração, sem DTO copiado à mão.

Fica mais difícil, e é aceito:

- **Cada tela do go-live é escrita duas vezes**, em React e em Flutter. Nenhum componente, teste ou regra
  de formulário atravessa de um front para o outro. É o maior custo desta decisão e foi escolhido de
  propósito: o dono do produto quer paridade, não um app reduzido.
- **O motor de sync entra no caminho crítico do go-live.** Versão, idempotência e pendência passam a ser
  requisito de toda rota de escrita da API, e o go-live não sai sem elas.
- **Dart entra no repo** como quarta linguagem de ferramental, com gate, lint e formatador próprios.
- **Loja entra no cronograma.** Conta Apple Developer (US$ 99 por ano) e Google Play Console (US$ 25, uma
  vez), revisão da Apple em cada publicação, e declaração de privacidade de dado de saúde nas duas lojas.
- **A API ganha um segundo transporte de sessão.** Cookie para o web, header para o app, com a mesma
  revogação.
- **Os três apps JS deixam de ser o repo inteiro**: a frase "os três apps usam npm" continua verdadeira
  para eles e deixa de descrever o `apps/mobile`.

Gatilho que reabre esta decisão: a paridade de telas custar mais do que o go-live aguenta, ou o
`swagger_parser` não gerar cliente utilizável do OpenAPI real da API — a saída, nesse caso, é o
`dart-dio` do `openapi-generator`.

## Alternatives considered

- **PWA com offline no `apps/web`** · rejeitada: o storage do Safari é despejável, não existe Background
  Sync no iOS, e dado de saúde ficaria em IndexedDB com a chave ao alcance do JavaScript.
- **Capacitor empacotando o `apps/web`** · rejeitada pelo dono do produto. Reaproveitaria as telas do web
  com SQLite e Keychain por plugin, mas continua sendo uma WebView, e o pedido foi app nativo.
- **Expo / React Native** · rejeitada: exige UI própria de qualquer forma, porque shadcn/ui e Radix não
  rodam fora do DOM, então não economiza a segunda escrita das telas que o Flutter também cobra.
- **Kotlin Multiplatform** · rejeitada: compartilha sync e banco, mas a UI é Compose Multiplatform ou
  duas UIs nativas, com mais configuração de build no iOS e ecossistema offline menor que o do
  `drift`.
- **Nativo por plataforma (Swift e Kotlin)** · rejeitada: sync, fila, criptografia e cada tela escritos
  duas vezes além do web — três fronts para um desenvolvedor só.
- **Last-write-wins no conflito** · rejeitada: sobrescreve evolução de outra pessoa e marca dois
  pacientes no mesmo horário sem ninguém saber, o que em prontuário é risco legal.
- **Base inteira da clínica no aparelho** · rejeitada: resolve qualquer consulta offline, mas põe todo o
  dado de saúde da clínica em cada celular, o pior caso de LGPD num aparelho perdido.

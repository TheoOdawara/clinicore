# 0014. Deploy de cada app pelo próprio Dockerfile no Coolify

- Status: accepted
- Date: 2026-10-15
- Emenda a: 0008 (o `compose.yaml` da raiz com seis serviços), issue #4 (o ambiente de homolog) e
  `AGENTS.md` da raiz (Arquitetura)

## Context

Homolog e produção vão rodar numa VPS com Coolify self-hosted. O plano registrado até aqui era um
`compose.yaml` na raiz com a stack inteira, publicado no Coolify como um recurso só.

O [`NFR-REL-04`](../requirements/non-functional/reliability.md#nfr-rel-04--publicação-sem-indisponibilidade)
pede que uma versão nova da API, do web e da landing seja publicada sem recusar requisição. O Coolify
só troca um container sem derrubar o anterior, por rolling update, em aplicação de Dockerfile,
Nixpacks, Railpack, Static ou imagem Docker. Num recurso Docker Compose, quem reconcilia é o
`docker compose up`, e o container antigo para antes de o novo subir
([Rolling updates](https://coolify.io/docs/knowledge-base/rolling-updates)).

Durante o rolling update, a versão antiga e a nova atendem ao mesmo tempo, sobre o mesmo banco. O
rolling update também deixa de valer quando o app publica porta no host ou tem nome de container fixo.

Duas regras já valem e continuam valendo. Primeira: a ordem do deploy é Postgres e Redis saudáveis,
depois a API, depois o front. Segunda: no dia a dia, uma dependência que cai nunca derruba a API; a
rota afetada responde erro e o processo fica de pé.

## Decision

- **Cada app é uma aplicação do Coolify com o próprio Dockerfile**, em `apps/<app>/Dockerfile`: a
  `api`, o `web` e o `site`. Não existe `compose.yaml` na raiz. O `apps/api/compose.yaml` continua
  sendo só o Postgres e o Redis de desenvolvimento.
- **O Postgres e o Redis são recursos Database do Coolify**, na mesma VPS, na rede interna, sem porta
  publicada no host. O backup agendado do Coolify é o caminho para a
  [OQ-11](../requirements/open-questions.md#oq-11--backup-e-recuperação), que continua em aberto.
- **Nenhuma aplicação publica porta no host nem fixa nome de container.** O tráfego entra pelo proxy do
  Coolify, que é o único caminho até a API.
- **A API aplica as migrations pendentes no boot, antes de abrir a porta.** Com o Postgres fora, ela
  espera e tenta de novo, sem sair. Uma migration que falha encerra o processo, e o Coolify mantém o
  container antigo. Esta é a única espera do boot: o Redis e os serviços externos continuam sob
  demanda.
- **O health check do Coolify aponta para o `/health`.** Como a porta só abre depois das migrations,
  um `/health` respondendo quer dizer schema migrado. O `/health` continua sem consultar dependência.
- **Toda migration é compatível com a versão anterior da API.** Mudança que quebra a versão em
  produção é feita em duas publicações: a primeira expande o schema e a segunda, depois que nenhum
  container antigo roda, contrai.
- **O `apps/web` é servido por nginx**, num Dockerfile multi-stage que copia o `dist/` do build do Vite
  e devolve o `index.html` para toda rota que não é arquivo.
- **O `apps/site` roda o `server.js` do `.next/standalone/`**, como a 0008 já definia, agora no próprio
  Dockerfile.
- **O worker, quando a #71 o criar, é outra aplicação do Coolify** com o Dockerfile da API e o binário
  do worker como comando.

## Consequences

Fica mais fácil:

- **Publicar no meio do expediente.** O container antigo só sai depois que o novo passa no health check,
  e um container novo que não sobe nunca recebe tráfego.
- **Publicar um app sem tocar nos outros.** Cada um tem build, versão e rollback próprios no Coolify.
- **Ler o que roda em produção.** O Dockerfile de cada app mora ao lado do código dele.

Fica mais difícil, e é aceito:

- **Não há um arquivo que sobe a stack inteira.** A ordem entre os recursos vem dos health checks e da
  espera da API pelo Postgres, não de um `depends_on`.
- **Migration destrutiva custa duas publicações.** Apagar ou renomear coluna em uso passa por expandir
  e contrair.
- **O boot da API passa a depender do Postgres para migrar.** Um restart com o Postgres fora deixa a API
  esperando, sem porta aberta, até ele voltar. O processo não morre, e a regra de que dependência que
  cai não derruba a API continua valendo depois do boot.
- **Duas versões rodam juntas por alguns segundos.** O que cada réplica faz sozinha, como a purga de
  hora em hora, precisa continuar idempotente.

## Alternatives considered

- **`compose.yaml` da raiz como um recurso do Coolify** · rejeitada: o Coolify não faz rolling update em
  Docker Compose, e cada publicação derrubaria a stack.
- **Migration no pre-deployment do Coolify** · rejeitada: o comando roda no container antigo, que não
  tem as migrations novas.
- **Migration no post-deployment do Coolify** · rejeitada: roda depois que o container novo já recebe
  tráfego, com código novo sobre schema velho.
- **Subcomando `api migrate` no comando do container** · rejeitada: com o Postgres fora num restart, o
  processo sai e o Docker o reinicia em loop.
- **Postgres e Redis em serviço gerenciado fora da VPS** · rejeitada: a VPS com Coolify já é o lugar
  da stack, e o recurso Database dele traz backup agendado.
- **`apps/web` pelo build pack Static do Coolify** · rejeitada: cada app é entregue pelo próprio
  Dockerfile, e o servidor do web fica versionado no repo.

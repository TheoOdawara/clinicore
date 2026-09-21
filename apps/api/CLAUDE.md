# Clinicore `apps/api` — camadas e pegadinhas

Aditivo ao `CLAUDE.md` da raiz e ao contrato global; em conflito, a raiz vence sobre este
arquivo apenas onde ela falar do mesmo assunto. A raiz tem a stack, os comandos, a visão geral da
arquitetura, as branches e o idioma.

## Camadas

**A dependência aponta para baixo: `Controller → Service → Repository → TypeORM`.** Quem a faz valer é
o container de DI, não convenção: o que não está nos `providers` do módulo não é alcançável.

- **Módulo** é o `<feature>.module.ts`, o único arquivo solto na raiz da feature. Declara
  controller, service e repository, registra as entities com `TypeOrmModule.forFeature()`, e
  exporta só o service quando outra feature precisa dele. `core.module.ts` é alcançado apenas pelo
  `AppModule`.
- **Subpasta existe quando tem conteúdo.** `enums/`, `constants/`, `utils/`, `job/` e `strategy/` só
  nascem com o primeiro arquivo; pasta vazia não é reservada.
- **Strategy do Passport é da feature, guard é de `common/`.** A `strategy/` da feature declara como um
  credencial vira usuário; o guard que a consome vale para toda feature, é registrado como `APP_GUARD`
  e mora em `common/guards/`, junto dos decorators que o acompanham.
- **Controller** é a classe `@Controller()` em `controller/`: rota, DTO de entrada e saída, chamada
  ao service. Não tem regra.
- **Service** é a regra de negócio, em `service/`. **Não importa `typeorm`, não usa
  `@InjectRepository` e não abre transação** — nem para uma consulta trivial. Quebrar isso é como o
  service vira arquivo de mil linhas.
- **Um service por operação ou grupo coeso de operações**, nunca um por feature. O nome diz qual:
  `protocol-archive.service.ts`, não um `protocol.service.ts` com vinte métodos públicos. Se os
  testes de um service precisam ser divididos por operação, o service já devia estar dividido.
- **Repository** é a classe `@Injectable()` em `repository/`, a única que injeta `EntityManager` ou
  `Repository<Entity>`, junto com `core/db/`. **Transação só aqui**: operação que grava em mais de
  uma tabela é um método de repository que abre o `dataSource.transaction()` dentro dele.
- **Entity** é a classe TypeORM em `entities/` da própria feature, uma por tabela.
- **A feature se divide quando acumula um segundo substantivo.** Anexo, nota e associação de um
  protocolo são features próprias, não subpastas dele. O gatilho é responsabilidade misturada; uma
  `dto/` com trinta arquivos é o alarme, não a regra.
- **DTO** é a classe com decorators de `class-validator` em `dto/`; o OpenAPI sai dela pelo plugin de
  CLI do `@nestjs/swagger`, sem `interface` paralela.
- **A URL é um recurso, nunca um verbo** (ADR 0006). Substantivo no plural e em kebab-case, e o método
  HTTP diz a operação: `GET` lê, `POST` cria, `PUT` substitui, `PATCH` altera em parte, `DELETE`
  remove. Operação sem nome natural vira sub-recurso substantivo (`POST /sessions/current/tokens`,
  `POST /password-resets/confirmation`). Token nunca vai no path, porque o logger registra o path. A
  única exceção é o OAuth, em `/oauth/<provedor>`.
- **Todo erro é Problem Details da RFC 9457** (ADR 0006), em `application/problem+json`, montado só
  pelo `business-error.filter.ts`: `type` é `tag:clinicore.com.br,2026:<código-em-kebab>` para o
  catálogo e `about:blank` para o resto, `title` fixo por `type`, `status`, e `errors` com
  `{ pointer, code }` só na validação.
- **Rota entregue é rota documentada.** O plugin cobre só o corpo da requisição; o resto é escrito no
  controller e faz parte da entrega, nunca de um passe depois: `@ApiOperation` com o resumo,
  `@ApiOkResponse` e companhia com o `type` da classe de resposta, **cada código de erro do catálogo
  que a rota pode devolver** por `@ApiProblemResponse`, que aponta para `common/dto/problem.response.ts`, e `headers` nas rotas que
  devolvem `Set-Cookie`. **A resposta é uma classe em `dto/`** — um `type` ou uma `interface` não
  chega ao documento, e é por isso que `SessionUserResponse` é classe.
- **`@ApiProperty` existe para o que o `class-validator` não conta**: regra de um decorator próprio
  (como a política de senha), exemplo e campo nulo. Onde o decorator padrão já diz, não se repete.
- **Não há dono de tabela.** O repository de uma feature lê e escreve a tabela que a operação dela
  precisa.
- **Só a API do TypeORM** (`find`, `findOne`, `relations`, `QueryBuilder`, `count` e afins). **Raw SQL é
  proibido, sem exceção**: `DataSource.query`, `EntityManager.query`, `QueryRunner.query` e qualquer
  trecho de SQL cru dentro de `QueryBuilder`. Agregação com join é composta no service a partir de
  chamadas separadas ao repository.
- **Relação vem por `relations`, e lote vem por `In()`.**
- **Erro:** o service lança `BusinessError` com um tipo (`NotFound`, `Conflict`, `Forbidden`,
  `Invalid`, `Unauthorized`) e um código; o repository traduz o erro conhecido do TypeORM — `QueryFailedError` com o
  código do Postgres, `EntityNotFoundError` — para esses tipos. Só o `ExceptionFilter` global em
  `common/filters/` conhece HTTP: converte o tipo em status e responde `{ code, message, fields }`.
  **`HttpException` do próprio framework mantém o status**, com o `code` sendo o nome do status em
  maiúsculas com sublinhado (`NOT_FOUND`), e só o que não é `BusinessError` nem `HttpException` vira
  500 sem detalhe, com a stack no Pino.
- **A `message` da API é inglês e é texto de desenvolvedor**, para log e depuração. O que o usuário lê
  é escrito no web, a partir do `code`.
- **Log:** `core/logger/` cria a instância do Pino e o `LoggerService` registrado por
  `app.useLogger()`, mais o **middleware** que registra a requisição. **Nenhum wrapper de terceiro.** O
  nível vem de `LOG_LEVEL`, obrigatória como toda variável. **O `/health` não é logado**, porque quem o
  chama é o orquestrador, a cada poucos segundos. **Nada de segredo sai no log:** header de
  autorização, cookie, senha e connection string passam pelo `redact` do Pino. O Pino escreve em
  `stdout`; o erro de ambiente do boot continua indo cru para `stderr`, antes de existir logger.
  **É middleware, não interceptor**: o guard roda antes do interceptor, então um interceptor não veria
  o `403` da `OriginGuard` nem o `404` de rota inexistente. O middleware registra o `res.on("finish")`
  e enxerga o status final, venha ele do controller, do guard ou do filtro. **O destino sai do
  `NODE_ENV`**: em `development` é o `pino-pretty`, colorido; em `test` e `production` é `stdout` cru.
  O `pino-pretty` é `devDependency` e é o formatador do próprio projeto Pino, não um wrapper de log.
- **Ambiente:** `core/config/` registra o `@nestjs/config` com uma classe validada por
  `class-validator` no boot. Toda variável é obrigatória, **sem default no ponto de leitura**. A
  leitura acontece por um acessor tipado, nunca por `ConfigService.get` direto. **O `class-validator`
  não reexporta `isIPRange`** — só `isIP` —, então o decorator de CIDR importa de `validator`, que é
  dependência direta da API por causa disso, ao lado de `@types/validator`.
- **Fila:** `core/queue/` registra o `@nestjs/bullmq` contra o Redis; o service enfileira pela fila
  injetada com `@InjectQueue`; o job é um `@Processor` em `features/<feature>/job/` e chama o service,
  como o controller faz. **O worker é um processo separado** (`worker.ts`), com o seu próprio
  container.
- **Testes** ficam em `__tests__/` da feature. O teste padrão é um por comportamento, subindo o módulo
  com `Test.createTestingModule` e batendo na rota com `supertest` contra o Postgres real. O e2e que
  sobe o `AppModule` inteiro fica em `test/`, com `<name>.e2e-spec.ts`, o `test/jest-e2e.json` e o
  gate `npm run test:e2e`. O `test/e2e-setup.ts` fixa o ambiente que os cenários de `Origin` e de log
  exigem, antes de o `AppModule` ser importado — o `ConfigModule.forRoot` lê o `process.env` no
  `require`, e mexer nele depois não muda nada. O destino do Pino é trocado por
  `overrideProvider(LOGGER)`, que é como o teste lê a linha emitida. Teste unitário existe só para cálculo puro (parcelamento,
  repasse). Não se faz mock de repository nem de `DataSource`.

## Pegadinhas da stack

Verificadas em 2026-09-19 contra as versões desta stack; as de `@nestjs/config`, `@nestjs/terminus`,
TypeScript e boot foram reconfirmadas em 2026-09-20, já com o código da #73 de pé.

- **O NestJS 12 é ESM-only** — o `@nestjs/common@12` publica `"type": "module"` e o 11 não. Um Jest em
  CommonJS não carrega ESM do `node_modules` e morre com `Must use import to load ES Module`. A saída
  seria `--experimental-vm-modules`, e o `vm.SourceTextModule` ainda é Stability 1 na documentação do
  Node 26, sem release alvo para sair. **O repo fica no Nest 11 por causa disso.**
- **Quem barra o TypeScript 7.0 na API é o `ts-jest@29`**, com peer `typescript >=4.3 <7`. O
  `@nestjs/cli@11.0.24` também carrega `typescript 5.9.3`, mas como dependência aninhada em
  `node_modules/@nestjs/cli/node_modules/`, e por isso não disputa com o `typescript` da raiz: o
  `6.0.3` instala e o `nest build` roda sobre ele. O `apps/web` também está no `6.0.3`.
- **O TypeScript 6.0 reprova `moduleResolution: "node"` e `baseUrl`** com `TS5107` e `TS5101`, a menos
  que se declare `ignoreDeprecations`. O jeito de continuar em CommonJS sem isso é
  `module` e `moduleResolution` em `node16`: o formato do emit vem do `type` do `package.json`, que
  não é `module`, e o `node16` é só o algoritmo de resolução moderno. **Trocar isso por ESM desfaz a
  razão de o repo estar no Nest 11.**
- **`ConfigService.get` devolve `any` sem `{ infer: true }`.** A sobrecarga que casa com
  `config.get('CHAVE')` tem `T = any`, e tipar o serviço como `ConfigService<Env, true>` não ajuda —
  `K` restringe só o nome da chave. `const port: number = config.get('PORT')` compila com `PORT`
  string. Por isso a leitura passa por um acessor tipado sobre `getOrThrow`, nunca por `get` direto.
- **Os pacotes-satélite do Nest saltaram a numeração para acompanhar o core**, e **nenhum deles entra
  na versão 12 enquanto o Jest for CommonJS.** O peer é `@nestjs/common: ^11.0.0 || ^12.0.0`, e é ele
  que engana: o peer aceita, o Jest não. O `@nestjs/config@12` e o `@nestjs/terminus@12` publicam
  `"type": "module"`, exatamente como o `@nestjs/common@12`. Os pins que rodam sobre o Nest 11 são
  `@nestjs/config@4.0.4`, `@nestjs/terminus@11.1.1` e `@nestjs/typeorm@11.0.3` — este último com o
  peer já aceitando `typeorm ^1.0.0-dev`, então o pin não custa versão do TypeORM. O `@nestjs/swagger`
  não é exceção, é o mesmo caso por outra porta: o `@nestjs/swagger@12` exige `@nestjs/common: ^12.0.0` e o npm recusa a
  instalação — o pin é `@nestjs/swagger@11.4.7`. **A conferência antes de subir um satélite é o
  `"type"` do `package.json` publicado, nunca o peer.**
- **O `ConfigModule.forRoot` do `@nestjs/config` é `async`** e devolve `Promise<DynamicModule>`. Um
  `validate` que lança rejeita essa promise no `require` do módulo que a chama, muito antes do
  `NestFactory`, e o erro sai como unhandled rejection ou como `[Nest] ERROR [ExceptionHandler]` com
  stack — nunca como a mensagem limpa que a spec exige. O boot que entrega
  `Invalid environment:` e as linhas de formato em `stderr` é
  `NestFactory.create(AppModule, { abortOnError: false, bufferLogs: true, autoFlushLogs: false })`
  com `try/catch` em volta: o `abortOnError: false` troca o `process.abort()` por um rethrow que
  chega ao `catch`, e o buffer sem flush automático retém o log que o `ExceptionsZone` escreve antes
  do teardown. No caminho feliz, `app.flushLogs()` solta os logs de boot do Nest.
- **O `ConfigModule.forRoot` lê o `.env` do diretório de trabalho por padrão**, então subir o binário
  de dentro de `apps/api` com uma variável faltando na linha de comando não falha: o `.env` do dev
  completa o que falta. Teste e verificação manual de ambiente inválido rodam com `cwd` fora do app.
- **O Swagger só é montado fora de produção**, pelo `NODE_ENV` do schema. A UI em `/api` e o
  documento em `/api-json` publicam o mapa de rotas, o formato dos DTO e as regras de validação.
- **O `app.enableShutdownHooks()` re-emite o sinal depois de fechar a aplicação**
  (`process.kill(process.pid, signal)`), então `SIGTERM` sai com **143**, nunca com 0. Fechamento
  limpo se confere pela ausência de `ERROR_DURING_SHUTDOWN` e pela porta liberada, não pelo código.
- **O `latest` do TypeORM é o 1.1.1, e o 0.3.x virou o dist-tag `legacy`.** Tutorial e resposta de
  fórum anteriores a isso descrevem a API do 0.3.
- **A CLI do TypeORM roda fora do Nest e ninguém carrega o `.env` para ela.** O `data-source.ts` lê
  `process.env` cru — é a licença da regra 6 — e falha nomeando a variável quando ela não está lá. Por
  isso os scripts `migration:*` invocam a CLI por `node --env-file-if-exists=.env`, que é nativo do
  Node e **deixa o ambiente já existente vencer**, então o job do CI entrega a `DATABASE_URL` sem
  disputa. O script `test` invoca o Jest pelo mesmo caminho, com `.env.test` no lugar do `.env`,
  então o app inteiro tem um mecanismo só de ambiente e nenhuma dependência para isso — o custo é que
  `npx jest` cru não enxerga arquivo nenhum, e quem fizer isso cai no `validateEnv` dos testes. **A
  flag não passa por `NODE_OPTIONS`** (`--env-file-if-exists= is not allowed`), e é por isso que o
  script invoca `node` com o binário do Jest como argumento em vez de chamar `jest` direto.
- **A suíte inteira exige o ambiente inteiro, não só a `DATABASE_URL`.** Subir o `ConfigModule` num
  teste roda o `validateEnv`, que cobra as cinco variáveis. Por isso o teste que precisa do ambiente
  chama `validateEnv(process.env)` no topo do módulo: a mensagem que falta uma variável chega no
  lugar de um erro de conexão ou de um teste com nome enganoso.
- **O `migration:generate` da CLI do TypeORM 1.1 exige o caminho do arquivo como argumento
  posicional**, então o script sozinho sai com `Argumentos insuficientes`. A forma que roda é
  `npm run migration:generate -- src/core/db/migrations/<Nome>`. O `migration:run` e o
  `migration:revert` são completos, e é essa assimetria que engana.
- **A partir da #74 o gate `npm run test` exige o Postgres do compose de pé.** O `DbModule` está no
  `CoreModule`, então o boot conecta — e os testes do `boot.test.ts` que sobem o processo real caem
  com `ECONNREFUSED` sem o container. Com `retryAttempts: 0` a falha é imediata, porque quem ordena a
  dependência é o `healthcheck` do compose, não retry às cegas.
- **O glob de `entities` casa `.ts` quando o `__dirname` é `src/`**, o que acontece sob o `ts-node` da
  CLI e sob o Jest. Com as entities da #66 no lugar, os três contextos — `dist`, `ts-node` e `ts-jest`
  — carregam a mesma lista, e é o que o `migration:run`, o `npm run test` e o `boot.test.ts` provam.
- **Declarar `enumName` numa coluna de enum cria drift permanente.** O loader do Postgres só devolve
  `enumName` quando o nome no banco difere do derivado (`<tabela>_<coluna>_enum`); com o nome igual ao
  derivado ele devolve `undefined`, e o comparador acha diferença contra a entity a cada geração. O
  arrasto é maior do que parece: trocar o tipo da coluna faz o gerador derrubar e recriar todo índice
  e unique que a usa. **A coluna de enum não declara nome.**
- **O gerador de migration sai com código 1 quando não há diferença**, com
  `No changes in database schema were found`. Por isso o gate de drift do CI roda com `--check`, que
  inverte isso: sai 0 só quando o schema bate com as entities, e 1 tanto em drift quanto em falha da
  própria CLI. Decidir pela existência do arquivo com `|| true` deixava a CLI quebrada passar como
  "sem drift".
- **As suítes de teste dividem o mesmo Postgres**, e a limpeza de uma derruba os dados da outra. Por
  isso o Jest da API roda com `maxWorkers: 1`; em paralelo, testes que passam isolados falham juntos.
- **O `delete({})` do TypeORM 1.1 é recusado** com `Empty criteria(s) are not allowed`. Apagar a tabela
  inteira é `createQueryBuilder().delete().from(Entity).execute()`.
- **A DI do Nest depende de `reflect-metadata` e de `emitDecoratorMetadata`.** Faltando qualquer um
  dos dois, a compilação passa e a injeção falha em runtime.
- **O `class-validator` não tem decorador de CIDR.** Tem `@IsIP`, `@IsPort`, `@IsUrl`, `@IsFQDN` e
  `@IsEmail`, mas nada de faixa — e `TRUSTED_PROXIES` precisa. O `validator` 13, que o próprio
  `class-validator` traz como dependência, tem `isIPRange`: o decorador sai de um `registerDecorator`
  de poucas linhas, nunca de regex à mão.
- **O `StandardSchemaValidationPipe` existe no `@nestjs/common@11` e não dá para usar.** Ele lê
  `metadata.schema`, e no 11 o `ArgumentMetadata` não tem esse campo nem o `@Body()` tem sobrecarga que
  o alimente. É encanamento adiantado para o 12. Conferir que o arquivo existe não basta.
- **A chave de contagem do throttler é um hash, não a rota.** O `@nest-lab/throttler-storage-redis`
  grava `{<sha256 de "<Controller>-<handler>-<throttler>-<tracker>">:<throttler>}:hits` e o par
  `:blocked`. Um `redis-cli --scan` por `/sessions` não acha nada; o teste calcula a chave pela mesma
  fórmula. Renomear um handler zera o contador dele.
- **O `quit()` do ioredis rejeita quando o cliente não está `ready`.** Com `enableOfflineQueue: false`,
  o Redis fora do ar faz o `quit()` falhar, e o shutdown sai com `ERROR_DURING_SHUTDOWN` e deixa o
  Jest preso na reconexão. O `RedisModule` só chama `quit()` com status `ready`; fora disso,
  `disconnect()`.

## Infraestrutura

- **O Gmail reescreve o `From` com a conta autenticada, em silêncio.** Mandar
  `from: "nao-responda@clinicore.app"` autenticado como outra conta não falha: chega com o endereço
  da conta. Alias do "Enviar e-mail como" não vale no `smtp.gmail.com`. Sair disso exige o
  `smtp-relay.gmail.com`, que é Google Workspace, não conta comum.
- **A autenticação é por app password, não pela senha da conta.** Criar uma exige verificação em duas
  etapas ligada, e o acesso a "apps menos seguros" acabou em 2025-05-01.
- **O limite de envio é diário e da conta inteira** — 2.000 mensagens por dia no Workspace, menos numa
  conta comum — e dev e homolog dividem o mesmo teto se apontarem para a mesma conta.

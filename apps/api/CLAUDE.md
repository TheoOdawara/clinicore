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
  Erro desconhecido vira 500 sem detalhe.
- **A `message` da API é inglês e é texto de desenvolvedor**, para log e depuração. O que o usuário lê
  é escrito no web, a partir do `code`.
- **Log:** `core/logger/` cria a instância do Pino e o `LoggerService` registrado por
  `app.useLogger()`, mais o interceptor que registra a requisição. **Nenhum wrapper de terceiro.** O
  nível vem de `LOG_LEVEL`, obrigatória como toda variável. **O `/health` não é logado**, porque quem o
  chama é o orquestrador, a cada poucos segundos. **Nada de segredo sai no log:** header de
  autorização, cookie, senha e connection string passam pelo `redact` do Pino. O Pino escreve em
  `stdout`; o erro de ambiente do boot continua indo cru para `stderr`, antes de existir logger.
- **Ambiente:** `core/config/` registra o `@nestjs/config` com uma classe validada por
  `class-validator` no boot. Toda variável é obrigatória, **sem default no ponto de leitura**. A
  leitura acontece por um acessor tipado, nunca por `ConfigService.get` direto.
- **Fila:** `core/queue/` registra o `@nestjs/bullmq` contra o Redis; o service enfileira pela fila
  injetada com `@InjectQueue`; o job é um `@Processor` em `features/<feature>/job/` e chama o service,
  como o controller faz. **O worker é um processo separado** (`worker.ts`), com o seu próprio
  container.
- **Testes** ficam em `__tests__/` da feature. O teste padrão é um por comportamento, subindo o módulo
  com `Test.createTestingModule` e batendo na rota com `supertest` contra o Postgres real. O e2e que
  sobe o `AppModule` inteiro fica em `test/`. Teste unitário existe só para cálculo puro (parcelamento,
  repasse). Não se faz mock de repository nem de `DataSource`.

## Pegadinhas da stack

Verificadas em 2026-09-19, contra as versões desta stack e antes de existir código em NestJS.

- **O NestJS 12 é ESM-only** — o `@nestjs/common@12` publica `"type": "module"` e o 11 não. Um Jest em
  CommonJS não carrega ESM do `node_modules` e morre com `Must use import to load ES Module`. A saída
  seria `--experimental-vm-modules`, e o `vm.SourceTextModule` ainda é Stability 1 na documentação do
  Node 26, sem release alvo para sair. **O repo fica no Nest 11 por causa disso.**
- **O TypeScript 7.0 está fora da API por duas vias.** O `@nestjs/cli@11.0.24` carrega
  `typescript 5.9.3` como dependência direta, e o `ts-jest@29` declara peer `typescript >=4.3 <7`. O
  `apps/web` segue no 7.0 porque usa `@swc/jest` e não passa por nenhum dos dois.
- **`ConfigService.get` devolve `any` sem `{ infer: true }`.** A sobrecarga que casa com
  `config.get('CHAVE')` tem `T = any`, e tipar o serviço como `ConfigService<Env, true>` não ajuda —
  `K` restringe só o nome da chave. `const port: number = config.get('PORT')` compila com `PORT`
  string. Por isso a leitura passa por um acessor tipado sobre `getOrThrow`, nunca por `get` direto.
- **Os pacotes-satélite do Nest saltaram a numeração para acompanhar o core.** O `@nestjs/config` foi
  de `4.0.4` para `12.0.0` sem nada entre os dois, e o mesmo vale para `@nestjs/schedule` e
  `@nestjs/event-emitter`. O peer deles é `@nestjs/common: ^11.0.0 || ^12.0.0`, então a versão 12
  desses pacotes roda sobre o Nest 11. **O `@nestjs/swagger` é a exceção e não generaliza:** o
  `@nestjs/swagger@12` exige `@nestjs/common: ^12.0.0` e o npm recusa a instalação sobre o Nest 11 —
  o pin é `@nestjs/swagger@11.4.7`. O `@nestjs/terminus@12` aceita as duas linhas e entra normal.
- **O `latest` do TypeORM é o 1.1.1, e o 0.3.x virou o dist-tag `legacy`.** Tutorial e resposta de
  fórum anteriores a isso descrevem a API do 0.3.
- **A DI do Nest depende de `reflect-metadata` e de `emitDecoratorMetadata`.** Faltando qualquer um
  dos dois, a compilação passa e a injeção falha em runtime.
- **O `class-validator` não tem decorador de CIDR.** Tem `@IsIP`, `@IsPort`, `@IsUrl`, `@IsFQDN` e
  `@IsEmail`, mas nada de faixa — e `TRUSTED_PROXIES` precisa. O `validator` 13, que o próprio
  `class-validator` traz como dependência, tem `isIPRange`: o decorador sai de um `registerDecorator`
  de poucas linhas, nunca de regex à mão.
- **O `StandardSchemaValidationPipe` existe no `@nestjs/common@11` e não dá para usar.** Ele lê
  `metadata.schema`, e no 11 o `ArgumentMetadata` não tem esse campo nem o `@Body()` tem sobrecarga que
  o alimente. É encanamento adiantado para o 12. Conferir que o arquivo existe não basta.

## Infraestrutura

- **O Gmail reescreve o `From` com a conta autenticada, em silêncio.** Mandar
  `from: "nao-responda@clinicore.app"` autenticado como outra conta não falha: chega com o endereço
  da conta. Alias do "Enviar e-mail como" não vale no `smtp.gmail.com`. Sair disso exige o
  `smtp-relay.gmail.com`, que é Google Workspace, não conta comum.
- **A autenticação é por app password, não pela senha da conta.** Criar uma exige verificação em duas
  etapas ligada, e o acesso a "apps menos seguros" acabou em 2025-05-01.
- **O limite de envio é diário e da conta inteira** — 2.000 mensagens por dia no Workspace, menos numa
  conta comum — e dev e homolog dividem o mesmo teto se apontarem para a mesma conta.

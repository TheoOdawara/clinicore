# `apps/mobile`

O sistema da clínica como app nativo de iOS e Android, em Flutter, com as mesmas telas do web e
funcionando sem conexão ([ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md)). Fala
com a API mandando `Clinicore-Client: mobile` e o token no `Authorization`.

**Nunca faz:** gerar código do OpenAPI da API, criar um segundo `Dio`, ler configuração fora do
`env.dart`.

## Blocos internos

| Bloco | Onde | Responsabilidade | Estado |
| --- | --- | --- | --- |
| Boot | `lib/main.dart` | lê a configuração e sobe o app | existe |
| App e rotas | `lib/app/` | o `MaterialApp` e o `GoRouter`, cujo `redirect` guarda a área autenticada | existe |
| Configuração | `lib/shared/env/` | lê e valida os valores do `--dart-define-from-file` | existe |
| Cliente HTTP | `lib/shared/http/` | a instância única do `Dio` | existe |
| Features | `lib/features/<feature>/` | telas, estado em Riverpod, mensagens e as chamadas ao `Dio` com o modelo escrito à mão | M1, a começar pela autenticação (#96 a #99) |
| Banco local | drift sobre SQLite cifrado | o dado offline, em duas camadas: ocupação da agenda e dado clínico | M1 |
| Fila de escrita | sobre o banco local | guarda cada escrita feita offline e a envia em ordem quando a conexão volta | M1 |
| Cofre | Keychain e Keystore | o token e a chave do banco local | M1 |

## Para mudar com segurança

As camadas e as pegadinhas estão no
[`apps/mobile/AGENTS.md`](https://github.com/TheoOdawara/clinicore/blob/staging/apps/mobile/AGENTS.md). Rota nova na API chega ao app quando a
feature escreve a chamada e o modelo dela à mão. O conceito de offline está em
[`../concepts/offline.md`](../concepts/offline.md).

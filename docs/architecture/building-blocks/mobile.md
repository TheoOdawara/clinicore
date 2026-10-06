# `apps/mobile`

O sistema da clínica como app nativo de iOS e Android, em Flutter, com as mesmas telas do web e
funcionando sem conexão ([ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md)). Fala
com a API mandando `Clinicore-Client: mobile` e o token no `Authorization`.

**Nunca faz:** editar o cliente gerado à mão, criar um segundo `Dio`, ler configuração fora do
`env.dart`.

## Blocos internos

| Bloco | Onde | Responsabilidade | Estado |
| --- | --- | --- | --- |
| Boot | `lib/main.dart` | lê a configuração e sobe o app | existe |
| App e rotas | `lib/app/` | o `MaterialApp` e o `GoRouter`, cujo `redirect` guarda a área autenticada | existe |
| Configuração | `lib/shared/env/` | lê e valida os valores do `--dart-define-from-file` | existe |
| Cliente HTTP | `lib/shared/http/` | a instância única do `Dio` | existe |
| Cliente da API | `lib/shared/api/` | o cliente Retrofit gerado do OpenAPI da API | existe, gerado |
| Features | `lib/features/<feature>/` | telas, estado e mensagens de uma feature | M1, a começar pela autenticação (#96 a #99) |
| Banco local | drift sobre SQLite cifrado | o dado offline, em duas camadas: ocupação da agenda e dado clínico | M1 |
| Fila de escrita | sobre o banco local | guarda cada escrita feita offline e a envia em ordem quando a conexão volta | M1 |
| Cofre | Keychain e Keystore | o token e a chave do banco local | M1 |

## Para mudar com segurança

As camadas e a receita de regeração do cliente estão no
[`apps/mobile/AGENTS.md`](../../../apps/mobile/AGENTS.md). Rota nova na API só chega ao app por essa
regeração, e o diff do cliente gerado vai no mesmo pull request que o usa. O conceito de offline está em
[`../concepts/offline.md`](../concepts/offline.md).

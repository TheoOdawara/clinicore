# Offline

**Marco: M1. Nada disto existe ainda.** A decisão é a
[ADR 0007](../../decisions/0007-app-nativo-em-flutter-com-offline.md), e os requisitos são `NFR-REL-02`,
`NFR-REL-03` e `NFR-SEC-04`.

O offline é exclusivo do `apps/mobile`. O `apps/web` exige conexão e não guarda dado clínico.

## No app

- **O app funciona igual sem conexão**, em leitura e em escrita.
- **Toda escrita vai para uma fila local**, em ordem, e sobe sozinha quando a conexão volta.
- **O dado no aparelho vem em duas camadas**: a ocupação da agenda, sem paciente, por um período longo;
  e o dado clínico só dos pacientes de um período curto. Os dois períodos são números da spec da agenda.
- **O banco local é cifrado**, com a chave no Keychain ou no Keystore.
- **Sem sincronizar há 72 horas, o app mostra 0 dados de saúde** e mantém a fila; no logout, apaga tudo.

## Na API

Toda rota de escrita que o app chama passa a exigir três coisas:

| O que | Para quê |
| --- | --- |
| Coluna de versão no registro | recusar uma escrita feita sobre versão velha |
| UUID gerado no cliente | identificar um registro criado offline |
| Chave de idempotência | aceitar o reenvio da fila sem duplicar |

## Conflito

**Nada é sobrescrito em silêncio.** Uma escrita da fila feita sobre versão velha, ou que ocupa um
horário já ocupado, é recusada pela API, e o item vai para "Pendências" no app, para a pessoa refazer.

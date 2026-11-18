# 0017. Fila de e-mail numa tabela do Postgres e worker com cron

- Status: accepted
- Date: 2026-10-20
- Completa: `0009-api-em-rust-com-axum-e-sqlx.md`, que deixou a fila e o agendador para a #71, e
  `0010-api-rs-com-crate-por-processo.md`, que previu o `crates/worker/`.

## Context

A API envia o e-mail com um `tokio::spawn` por mensagem, sem retry. Um restart ou uma falha passageira
do SMTP perde o link de confirmação ou de redefinição, e a pessoa só percebe que ele não chegou. A
purga do que venceu roda de hora em hora dentro de cada réplica da API, quando a regra 17 da spec 003
pede uma execução por dia, às 03:00 de São Paulo.

O e-mail carrega um token cujo hash é gravado numa transação. Se o token e o pedido de envio vivem em
lugares diferentes, um pode existir sem o outro: o token gravado sem e-mail, ou o e-mail pedido para um
token que a transação desfez.

O Rust não tem um par oficial do BullMQ. As opções reais são uma fila sobre o Redis (`apalis`), uma
fila sobre o Postgres, à mão ou por crate, e o agendador num crate de cron.

## Decision

**A fila de e-mail é a tabela `email_outbox`, gravada pela API na mesma transação do token.** O
`crates/worker/` (pacote `clinicore-worker`, binário `worker`) é o único processo que fala com o SMTP:
ele reserva as linhas pendentes com `FOR UPDATE SKIP LOCKED`, envia, e apaga a linha enviada.

- **A API renderiza o e-mail e grava assunto, HTML e texto.** O worker não conhece template, link nem
  feature, e por isso não precisa de `APP_ORIGIN`.
- **O link vive em texto puro na linha só enquanto ela pode ser enviada.** A linha enviada é apagada,
  a que esgota as tentativas perde os dois corpos na mesma escrita que a marca `failed`, e nenhuma é
  enviada depois do `expires_at`, que é o do token.
- **A entrega é pelo menos uma vez.** O worker que cai entre o envio e o `DELETE` deixa a linha
  reservada, e ela volta a ser enviada quando a reserva vence.
- **A purga roda no worker por cron**, `0 3 * * *` em `America/Sao_Paulo`, com `croner` sobre o
  `jiff`, e o banco de fusos embutido no binário (`tzdb-bundle-always`), para que o container não
  dependa do `tzdata` da imagem.
- **Duas instâncias do worker rodam a purga duas vezes, e isso é aceito.** O `DELETE` repetido apaga
  zero linhas, então não há lock nem eleição de líder.

## Consequences

Fica mais fácil:

- O e-mail sobrevive a restart, deploy e SMTP fora do ar, e o retry tem teto e backoff.
- A API deixa de falar com o SMTP e de exigir `SMTP_*` e `MAIL_FROM`.
- Não há fila para operar fora do Postgres, e o Redis continua guardando só o que pode se perder.

Fica mais difícil, e é aceito:

- **O link de uso único fica no banco em texto puro por até uma hora.** Quem lê o banco já lê muito
  mais do que isso; a linha some no envio, e o corpo some na desistência.
- **O e-mail sai com o atraso do polling**, de até 5 segundos, mais o tempo do SMTP.
- **Um e-mail pode chegar duas vezes**, quando o worker cai entre enviar e apagar. Os dois links são
  o mesmo token.
- **Uma purga perdida espera o dia seguinte.** O worker parado às 03:00 não recupera a execução; a
  tabela cresce um dia a mais.

## Alternatives considered

- **`apalis` sobre o Redis** · rejeitada: a gravação do token e do pedido de envio deixa de ser
  atômica, e o Redis passa a guardar estado que não pode se perder.
- **`apalis` sobre o Postgres** · rejeitada: traz o modelo de job genérico, a tabela e as migrations
  dele para um único tipo de mensagem que uma consulta resolve.
- **Lock consultivo para uma purga por dia** · rejeitada: protege contra um efeito que não existe,
  porque a segunda execução não apaga nada.
- **`tokio::time::interval` em vez de cron** · rejeitada: a regra 17 fixa o horário e o fuso.
- **`cron` sobre `chrono` e `chrono-tz`** · rejeitada: o `jiff` já está no lockfile, e o `croner`
  aceita a expressão de cinco campos da regra como ela está escrita.
- **O worker emitir o token** · rejeitada: leva emissão de segredo, link e template para o worker, e
  repete nele as consultas de duas features.

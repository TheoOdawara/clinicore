use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clinicore_worker::mail::{Mailer, Transport};
use clinicore_worker::outbox::{self, claim, deliver};
use lettre::transport::stub::AsyncStubTransport;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use sqlx::PgPool;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::support::{capture_log, count};

const EMAIL: &str = "ana@exemplo.com";

async fn enqueue(pool: &PgPool, email: &str, attempts: i32, expires_in_seconds: f64) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO email_outbox (email, subject, html_body, text_body, attempts, expires_at)
        VALUES ($1, 'Confirm your email', '<p>html body</p>', 'text body', $2,
            now() + make_interval(secs => $3))
        RETURNING id",
    )
    .bind(email)
    .bind(attempts)
    .bind(expires_in_seconds)
    .fetch_one(pool)
    .await
    .expect("a queued mail")
}

fn stub_mailer(stub: &AsyncStubTransport) -> Mailer {
    Mailer::new("clinicore@example.com", Transport::Stub(stub.clone())).expect("a valid sender")
}

async fn process(pool: &PgPool, mailer: &Mailer) {
    for mail in claim(pool).await.expect("a claim") {
        deliver(pool, mailer, mail)
            .await
            .expect("a recorded delivery");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_delivered_mail_leaves_the_queue(pool: PgPool) {
    enqueue(&pool, EMAIL, 0, 3600.0).await;
    let stub = AsyncStubTransport::new_ok();

    process(&pool, &stub_mailer(&stub)).await;

    let messages = stub.messages().await;
    assert_eq!(messages.len(), 1);
    let (envelope, raw) = &messages[0];
    assert_eq!(envelope.to()[0].to_string(), EMAIL);
    for part in [
        "Subject: Confirm your email",
        "<p>html body</p>",
        "text body",
    ] {
        assert!(raw.contains(part), "{part}");
    }
    assert_eq!(count(&pool, "SELECT count(*) FROM email_outbox").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_delivery_backs_off_and_the_fifth_gives_up(pool: PgPool) {
    let (log, _guard) = capture_log();
    let mailer = stub_mailer(&AsyncStubTransport::new_error());
    let first_failure = enqueue(&pool, EMAIL, 0, 3600.0).await;

    process(&pool, &mailer).await;

    let retrying: (String, i32, bool, bool) = sqlx::query_as(
        "SELECT status::text, attempts, html_body IS NOT NULL AND text_body IS NOT NULL,
            next_attempt_at BETWEEN now() + interval '55 seconds' AND now() + interval '1 minute'
        FROM email_outbox WHERE id = $1",
    )
    .bind(first_failure)
    .fetch_one(&pool)
    .await
    .expect("the retrying mail");
    assert_eq!(retrying, ("pending".to_string(), 1, true, true));
    assert!(
        log.lines().iter().any(|line| {
            line["level"] == "WARN"
                && line["message"] == "mail delivery failed, retrying"
                && line["email"] == "a***@exemplo.com"
                && line["attempt"] == 1
                && line["reason"] == "stub error"
        }),
        "{:?}",
        log.lines()
    );

    sqlx::query("DELETE FROM email_outbox")
        .execute(&pool)
        .await
        .expect("an empty queue");
    enqueue(&pool, EMAIL, 4, 3600.0).await;

    process(&pool, &mailer).await;

    let gave_up = "SELECT count(*) FROM email_outbox
        WHERE status = 'failed' AND attempts = 5 AND html_body IS NULL AND text_body IS NULL";
    assert_eq!(count(&pool, gave_up).await, 1);
    assert!(
        log.lines().iter().any(|line| {
            line["level"] == "ERROR"
                && line["message"] == "mail delivery gave up"
                && line["email"] == "a***@exemplo.com"
                && line["attempts"] == 5
                && line["reason"] == "stub error"
        }),
        "{:?}",
        log.lines()
    );
    sqlx::query("UPDATE email_outbox SET next_attempt_at = now() - interval '1 second'")
        .execute(&pool)
        .await
        .expect("a due row");
    assert!(claim(&pool).await.expect("a claim").is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_expired_mail_is_never_sent(pool: PgPool) {
    enqueue(&pool, EMAIL, 0, -1.0).await;
    let stub = AsyncStubTransport::new_ok();

    process(&pool, &stub_mailer(&stub)).await;

    assert!(stub.messages().await.is_empty());
    let untouched = "SELECT count(*) FROM email_outbox WHERE status = 'pending' AND attempts = 0";
    assert_eq!(count(&pool, untouched).await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn concurrent_claims_split_the_queue_and_an_abandoned_claim_comes_back(pool: PgPool) {
    for index in 0..10 {
        enqueue(&pool, &format!("pessoa{index}@exemplo.com"), 0, 3600.0).await;
    }

    let (first, second) = tokio::join!(claim(&pool), claim(&pool));

    let first: HashSet<Uuid> = first.expect("a claim").iter().map(|mail| mail.id).collect();
    let second: HashSet<Uuid> = second
        .expect("a claim")
        .iter()
        .map(|mail| mail.id)
        .collect();
    assert!(first.is_disjoint(&second));
    assert_eq!(first.len() + second.len(), 10);
    let claimed = "SELECT count(*) FROM email_outbox WHERE attempts = 1
        AND next_attempt_at BETWEEN now() + interval '115 seconds' AND now() + interval '2 minutes'";
    assert_eq!(count(&pool, claimed).await, 10);

    let abandoned = *first.union(&second).next().expect("a claimed row");
    sqlx::query(
        "UPDATE email_outbox SET next_attempt_at = now() - interval '1 second' WHERE id = $1",
    )
    .bind(abandoned)
    .execute(&pool)
    .await
    .expect("an expired claim");

    let reclaimed = claim(&pool).await.expect("a claim");
    assert_eq!(reclaimed.len(), 1);
    assert_eq!((reclaimed[0].id, reclaimed[0].attempts), (abandoned, 2));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_stop_finishes_the_mail_in_flight_and_claims_nothing_else(pool: PgPool) {
    let (port, received, in_flight) = slow_smtp().await;
    enqueue(&pool, EMAIL, 0, 3600.0).await;
    let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("127.0.0.1")
        .port(port)
        .build();
    let mailer =
        Mailer::new("clinicore@example.com", Transport::Smtp(transport)).expect("a valid sender");
    let stop = CancellationToken::new();
    let worker = tokio::spawn(outbox::run(pool.clone(), mailer, stop.clone()));

    in_flight.await.expect("the mail in flight");
    enqueue(&pool, "bia@exemplo.com", 0, 3600.0).await;
    stop.cancel();
    tokio::time::timeout(Duration::from_secs(5), worker)
        .await
        .expect("a stopped worker")
        .expect("a worker that did not panic");

    let data = received.lock().expect("an unpoisoned message").clone();
    assert!(data.contains(&format!("To: {EMAIL}")), "{data}");
    let sent = "SELECT count(*) FROM email_outbox WHERE email = 'ana@exemplo.com'";
    assert_eq!(count(&pool, sent).await, 0);
    let unclaimed =
        "SELECT count(*) FROM email_outbox WHERE email = 'bia@exemplo.com' AND attempts = 0";
    assert_eq!(count(&pool, unclaimed).await, 1);
}

async fn slow_smtp() -> (u16, Arc<Mutex<String>>, oneshot::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("a free port");
    let port = listener.local_addr().expect("a bound address").port();
    let received = Arc::new(Mutex::new(String::new()));
    let message = received.clone();
    let (arrived, in_flight) = oneshot::channel();
    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.expect("a client");
        let (reader, mut writer) = socket.into_split();
        let mut lines = BufReader::new(reader).lines();
        let mut arrived = Some(arrived);
        let mut in_data = false;
        writer
            .write_all(b"220 localhost\r\n")
            .await
            .expect("a greeting");
        while let Some(line) = lines.next_line().await.expect("a line") {
            let reply: &[u8] = if in_data && line == "." {
                in_data = false;
                if let Some(arrived) = arrived.take() {
                    arrived.send(()).expect("a waiting test");
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
                b"250 queued\r\n"
            } else if in_data {
                let mut message = message.lock().expect("an unpoisoned message");
                message.push_str(&line);
                message.push('\n');
                continue;
            } else if line == "DATA" {
                in_data = true;
                b"354 go ahead\r\n"
            } else if line == "QUIT" {
                b"221 bye\r\n"
            } else {
                b"250 ok\r\n"
            };
            writer.write_all(reply).await.expect("a reply");
        }
    });
    (port, received, in_flight)
}

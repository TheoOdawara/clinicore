use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use clinicore_worker::outbox::{claim, cycle};
use sqlx::PgPool;
use tokio::sync::watch;
use uuid::Uuid;

use crate::support::{Behavior, Gate, MAIL_FROM, capture_log, mailer, queue, smtp};

fn running() -> watch::Receiver<bool> {
    watch::channel(false).1
}

async fn row(pool: &PgPool, id: Uuid) -> Option<(String, i32, bool, bool)> {
    sqlx::query_as(
        "SELECT status::text, attempts, html_body IS NULL AND text_body IS NULL,
             next_attempt_at BETWEEN now() + interval '59 seconds' AND now() + interval '1 minute'
         FROM email_outbox WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .expect("the queue is readable")
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_queued_mail_is_sent_and_deleted(pool: PgPool) {
    let smtp = smtp(Behavior::default()).await;
    let id = queue(&pool, "ana@exemplo.com").await;

    cycle(&pool, &mailer(&smtp, &[]), &running()).await;

    let received = smtp.received();
    assert_eq!(received.len(), 1);
    let message = &received[0];
    assert!(message.contains("To: ana@exemplo.com"), "{message}");
    assert!(message.contains(&format!("From: {MAIL_FROM}")), "{message}");
    assert!(message.contains("Subject: Confirm your email"), "{message}");
    assert!(message.contains("multipart/alternative"), "{message}");
    assert!(message.contains("Text body"), "{message}");
    assert!(message.contains("<p>Html body</p>"), "{message}");
    assert_eq!(row(&pool, id).await, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_redirect_sends_every_mail_to_one_address(pool: PgPool) {
    let smtp = smtp(Behavior::default()).await;
    queue(&pool, "ana@exemplo.com").await;

    let mailer = mailer(&smtp, &[("MAIL_REDIRECT_TO", "dev@exemplo.com")]);
    cycle(&pool, &mailer, &running()).await;

    let received = smtp.received();
    assert!(
        received[0].contains("To: dev@exemplo.com"),
        "{}",
        received[0]
    );
    assert!(!received[0].contains("ana@exemplo.com"), "{}", received[0]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failure_retries_with_backoff_and_the_fifth_gives_up(pool: PgPool) {
    let smtp = smtp(Behavior {
        refuse: true,
        ..Behavior::default()
    })
    .await;
    let mailer = mailer(&smtp, &[]);
    let id = queue(&pool, "ana@exemplo.com").await;
    let (log, _guard) = capture_log();

    cycle(&pool, &mailer, &running()).await;

    assert_eq!(
        row(&pool, id).await,
        Some(("pending".to_owned(), 1, false, true))
    );
    let text = log.text();
    assert!(text.contains("mail delivery failed, retrying"), "{text}");
    assert!(text.contains(r#""email":"a***@exemplo.com""#), "{text}");
    assert!(text.contains(r#""attempt":1"#), "{text}");
    assert!(!text.contains("ana@exemplo.com"), "{text}");

    sqlx::query(
        "UPDATE email_outbox SET attempts = 4, next_attempt_at = now() - interval '1 second'",
    )
    .execute(&pool)
    .await
    .expect("the row is due");
    cycle(&pool, &mailer, &running()).await;

    let (status, attempts, bodies_gone, _) = row(&pool, id).await.expect("the row stays");
    assert_eq!(
        (status.as_str(), attempts, bodies_gone),
        ("failed", 5, true)
    );
    let text = log.text();
    assert!(text.contains(r#""level":"ERROR""#), "{text}");
    assert!(text.contains("mail delivery gave up"), "{text}");
    assert!(text.contains(r#""attempts":5"#), "{text}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_expired_mail_is_not_sent(pool: PgPool) {
    let smtp = smtp(Behavior::default()).await;
    let id = queue(&pool, "ana@exemplo.com").await;
    sqlx::query("UPDATE email_outbox SET expires_at = now() - interval '1 second'")
        .execute(&pool)
        .await
        .expect("the row expires");

    cycle(&pool, &mailer(&smtp, &[]), &running()).await;

    assert!(smtp.received().is_empty());
    let (status, attempts, _, _) = row(&pool, id).await.expect("the row stays");
    assert_eq!((status.as_str(), attempts), ("pending", 0));
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_workers_never_claim_the_same_mail_and_a_stale_claim_returns(pool: PgPool) {
    for index in 0..10 {
        queue(&pool, &format!("pessoa{index}@exemplo.com")).await;
    }

    let (first, second) = tokio::join!(claim(&pool), claim(&pool));
    let ids: Vec<Uuid> = first
        .expect("the claim runs")
        .into_iter()
        .chain(second.expect("the claim runs"))
        .map(|mail| mail.id)
        .collect();

    assert_eq!(ids.len(), 10);
    assert_eq!(ids.iter().collect::<HashSet<_>>().len(), 10);
    assert!(claim(&pool).await.expect("the claim runs").is_empty());

    sqlx::query(
        "UPDATE email_outbox SET next_attempt_at = now() - interval '1 second' WHERE id = $1",
    )
    .bind(ids[0])
    .execute(&pool)
    .await
    .expect("the claim goes stale");
    let again = claim(&pool).await.expect("the claim runs");

    assert_eq!(again.len(), 1);
    assert_eq!((again[0].id, again[0].attempts), (ids[0], 2));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_stop_lets_the_mail_in_flight_finish_and_claims_nothing_after(pool: PgPool) {
    let gate = Arc::new(Gate::default());
    let smtp = smtp(Behavior {
        hold: Some(gate.clone()),
        ..Behavior::default()
    })
    .await;
    let first = queue(&pool, "ana@exemplo.com").await;
    let second = queue(&pool, "bia@exemplo.com").await;
    sqlx::query(
        "UPDATE email_outbox SET next_attempt_at = now() - interval '1 minute' WHERE id = $1",
    )
    .bind(first)
    .execute(&pool)
    .await
    .expect("the first row is due earlier");
    let (stop, stopped) = watch::channel(false);
    let worker = tokio::spawn(clinicore_worker::run(
        pool.clone(),
        mailer(&smtp, &[]),
        stopped,
    ));

    gate.reached.notified().await;
    stop.send_replace(true);
    gate.release.notify_one();
    tokio::time::timeout(Duration::from_secs(5), worker)
        .await
        .expect("the worker stops")
        .expect("the worker does not panic");

    assert_eq!(smtp.received().len(), 1);
    assert_eq!(row(&pool, first).await, None);
    let (status, _, _, _) = row(&pool, second)
        .await
        .expect("the second row stays queued");
    assert_eq!(status, "pending");
    let third = queue(&pool, "cid@exemplo.com").await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let (_, attempts, _, _) = row(&pool, third).await.expect("the third row stays queued");
    assert_eq!(attempts, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_claim_that_fails_is_logged(pool: PgPool) {
    let smtp = smtp(Behavior::default()).await;
    let mailer = mailer(&smtp, &[]);
    pool.close().await;
    let (log, _guard) = capture_log();

    cycle(&pool, &mailer, &running()).await;

    assert!(
        log.text().contains("could not claim the pending mail"),
        "{}",
        log.text()
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_result_that_cannot_be_recorded_is_logged(pool: PgPool) {
    let gate = Arc::new(Gate::default());
    let smtp = smtp(Behavior {
        hold: Some(gate.clone()),
        ..Behavior::default()
    })
    .await;
    let mailer = mailer(&smtp, &[]);
    queue(&pool, "ana@exemplo.com").await;
    let (log, _guard) = capture_log();

    let closing = pool.clone();
    let stop = running();
    let sending = cycle(&pool, &mailer, &stop);
    let breaking = async {
        gate.reached.notified().await;
        closing.close().await;
        gate.release.notify_one();
    };
    tokio::join!(sending, breaking);

    assert!(
        log.text().contains("could not record the mail result"),
        "{}",
        log.text()
    );
}

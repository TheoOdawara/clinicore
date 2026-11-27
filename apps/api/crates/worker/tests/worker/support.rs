use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

use clinicore_core::config::WorkerConfig;
use clinicore_worker::mail::Mailer;
use sqlx::PgPool;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::fmt::MakeWriter;
use uuid::Uuid;

pub const MAIL_FROM: &str = "Clinicore <nao-responda@clinicore.com.br>";

#[derive(Default)]
pub struct Gate {
    pub reached: Notify,
    pub release: Notify,
}

#[derive(Clone, Default)]
pub struct Behavior {
    pub refuse: bool,
    pub hold: Option<Arc<Gate>>,
}

pub struct Smtp {
    pub port: u16,
    received: Arc<Mutex<Vec<String>>>,
}

impl Smtp {
    pub fn received(&self) -> Vec<String> {
        self.received
            .lock()
            .expect("the inbox is not poisoned")
            .clone()
    }
}

pub async fn smtp(behavior: Behavior) -> Smtp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a local port is free");
    let port = listener.local_addr().expect("the port is bound").port();
    let received = Arc::new(Mutex::new(Vec::new()));
    let inbox = received.clone();
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            tokio::spawn(session(socket, behavior.clone(), inbox.clone()));
        }
    });
    Smtp { port, received }
}

async fn session(socket: TcpStream, behavior: Behavior, inbox: Arc<Mutex<Vec<String>>>) {
    let (read, mut write) = socket.into_split();
    let mut lines = BufReader::new(read).lines();
    let _ = write.write_all(b"220 fake ESMTP\r\n").await;
    while let Ok(Some(line)) = lines.next_line().await {
        let command = line.to_ascii_uppercase();
        let reply: &[u8] = if command.starts_with("EHLO") {
            b"250-fake\r\n250-AUTH PLAIN LOGIN\r\n250 8BITMIME\r\n"
        } else if command.starts_with("AUTH") {
            b"235 authenticated\r\n"
        } else if command == "DATA" {
            let _ = write.write_all(b"354 go ahead\r\n").await;
            let mut message = Vec::new();
            while let Ok(Some(line)) = lines.next_line().await {
                if line == "." {
                    break;
                }
                message.push(line);
            }
            if let Some(gate) = &behavior.hold {
                gate.reached.notify_one();
                gate.release.notified().await;
            }
            if behavior.refuse {
                b"554 refused\r\n"
            } else {
                inbox
                    .lock()
                    .expect("the inbox is not poisoned")
                    .push(message.join("\n"));
                b"250 queued\r\n"
            }
        } else if command == "QUIT" {
            let _ = write.write_all(b"221 bye\r\n").await;
            return;
        } else {
            b"250 ok\r\n"
        };
        let _ = write.write_all(reply).await;
    }
}

pub fn config(port: u16, overrides: &[(&str, &str)]) -> WorkerConfig {
    let port = port.to_string();
    let mut source = HashMap::from([
        ("APP_ENV", "test"),
        ("LOG_LEVEL", "warn"),
        (
            "DATABASE_URL",
            "postgresql://clinicore@127.0.0.1:1/clinicore",
        ),
        ("MAIL_FROM", MAIL_FROM),
        ("MAIL_REDIRECT_TO", "off"),
        ("SMTP_HOST", "localhost"),
        ("SMTP_PORT", port.as_str()),
        ("SMTP_TLS", "none"),
        ("SMTP_USER", "clinicore"),
        ("SMTP_PASSWORD", "a-test-password"),
    ]);
    source.extend(overrides.iter().copied());
    WorkerConfig::from_source(|name| source.get(name).map(|value| (*value).to_owned()))
        .expect("the test environment is valid")
}

pub fn mailer(smtp: &Smtp, overrides: &[(&str, &str)]) -> Mailer {
    Mailer::new(&config(smtp.port, overrides)).expect("the test mailer builds")
}

pub async fn queue(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO email_outbox (email, subject, html_body, text_body, expires_at)
         VALUES ($1, 'Confirm your email', '<p>Html body</p>', 'Text body', now() + interval '1 hour')
         RETURNING id",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("the mail is queued")
}

#[derive(Clone, Default)]
pub struct Captured(Arc<Mutex<Vec<u8>>>);

impl Captured {
    pub fn text(&self) -> String {
        String::from_utf8(self.0.lock().expect("the buffer is not poisoned").clone())
            .expect("the log is UTF-8")
    }
}

impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("the buffer is not poisoned")
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Captured {
    type Writer = Captured;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

pub fn capture_log() -> (Captured, DefaultGuard) {
    let captured = Captured::default();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_writer(captured.clone())
        .finish();
    (
        captured.clone(),
        tracing::subscriber::set_default(subscriber),
    )
}

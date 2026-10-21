use std::io::{self, Write};
use std::sync::{Arc, Mutex, Once};

use clinicore_core::config::AppEnv;
use sqlx::PgPool;
use tracing::level_filters::LevelFilter;
use tracing::subscriber::{DefaultGuard, NoSubscriber};

pub async fn count(pool: &PgPool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql)
        .fetch_one(pool)
        .await
        .expect("a count")
}

pub fn capture_log() -> (CapturedLog, DefaultGuard) {
    static SHARED_DISPATCHER: Once = Once::new();
    SHARED_DISPATCHER.call_once(|| {
        tracing::subscriber::set_global_default(NoSubscriber::default())
            .expect("no other global subscriber in the test binary");
    });
    let log = CapturedLog::default();
    let writer = log.clone();
    let guard = tracing::subscriber::set_default(clinicore_core::telemetry::subscriber(
        AppEnv::Test,
        LevelFilter::INFO,
        move || writer.clone(),
    ));
    (log, guard)
}

#[derive(Clone, Default)]
pub struct CapturedLog(Arc<Mutex<Vec<u8>>>);

impl CapturedLog {
    pub fn lines(&self) -> Vec<serde_json::Value> {
        let bytes = self.0.lock().expect("an unpoisoned log").clone();
        String::from_utf8(bytes)
            .expect("a UTF-8 log")
            .lines()
            .map(|line| serde_json::from_str(line).expect("a JSON log line"))
            .collect()
    }
}

impl Write for CapturedLog {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("an unpoisoned log")
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

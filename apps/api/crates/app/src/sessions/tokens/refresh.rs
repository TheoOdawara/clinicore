use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use utoipa::openapi::schema::Object;
use uuid::Uuid;

use crate::credentials::secret;
use crate::http::client::SessionClient;
use crate::http::openapi;

pub const ABSOLUTE_LIFETIME: Duration = Duration::from_secs(30 * 24 * 60 * 60);

pub static FORMAT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^[0-9a-f]{{8}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{12}}\.{}$",
        secret::PATTERN
    ))
    .expect("a valid pattern")
});

pub fn schema() -> Object {
    openapi::matching(&FORMAT)
}

pub fn lifetime(client: SessionClient) -> Duration {
    match client {
        SessionClient::Web => Duration::from_secs(24 * 60 * 60),
        SessionClient::Mobile => Duration::from_secs(7 * 24 * 60 * 60),
    }
}

pub fn longest_lifetime() -> Duration {
    lifetime(SessionClient::Web).max(lifetime(SessionClient::Mobile))
}

pub fn compose(session_id: Uuid, secret: &str) -> String {
    format!("{session_id}.{secret}")
}

pub fn parse(raw: &str) -> Option<(Uuid, &str)> {
    if !FORMAT.is_match(raw) {
        return None;
    }
    let (session_id, secret) = raw.split_once('.')?;
    Uuid::try_parse(session_id).ok().map(|id| (id, secret))
}

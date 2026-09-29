use std::time::Duration;

use uuid::Uuid;

use crate::http::client::SessionClient;

pub const ABSOLUTE_LIFETIME: Duration = Duration::from_secs(30 * 24 * 60 * 60);

pub fn lifetime(client: SessionClient) -> Duration {
    match client {
        SessionClient::Web => Duration::from_secs(24 * 60 * 60),
        SessionClient::Mobile => Duration::from_secs(7 * 24 * 60 * 60),
    }
}

pub fn compose(session_id: Uuid, secret: &str) -> String {
    format!("{session_id}.{secret}")
}

pub fn parse(raw: &str) -> Option<(Uuid, &str)> {
    let (session_id, secret) = raw.split_once('.')?;
    let is_secret = secret.len() == 43
        && secret
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if !is_secret {
        return None;
    }
    Uuid::try_parse(session_id).ok().map(|id| (id, secret))
}

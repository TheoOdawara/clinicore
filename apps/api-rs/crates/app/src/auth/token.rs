use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::error::AuthError;

pub struct IssuedToken {
    pub secret: String,
    pub hash: String,
}

pub fn issue() -> Result<IssuedToken, AuthError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)?;
    let secret = URL_SAFE_NO_PAD.encode(bytes);
    Ok(IssuedToken {
        hash: hash(&secret),
        secret,
    })
}

pub fn hash(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

pub fn refresh_token(session_id: Uuid, secret: &str) -> String {
    format!("{session_id}.{secret}")
}

pub fn parse_refresh_token(raw: &str) -> Option<(Uuid, &str)> {
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

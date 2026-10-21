use std::sync::LazyLock;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use regex::Regex;
use sha2::{Digest, Sha256};
use utoipa::openapi::schema::Object;

use super::error::CredentialError;
use crate::http::openapi;

pub const PATTERN: &str = "[A-Za-z0-9_-]{43}";

pub static FORMAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("^{PATTERN}$")).expect("a valid pattern"));

pub fn schema() -> Object {
    openapi::matching(&FORMAT)
}

pub struct IssuedSecret {
    pub secret: String,
    pub hash: String,
}

pub fn issue() -> Result<IssuedSecret, CredentialError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)?;
    let secret = URL_SAFE_NO_PAD.encode(bytes);
    Ok(IssuedSecret {
        hash: hash(&secret),
        secret,
    })
}

pub fn hash(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

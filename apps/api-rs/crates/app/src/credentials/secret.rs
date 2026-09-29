use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use super::error::CredentialError;

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

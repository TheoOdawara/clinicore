use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use crate::error::AppError;

pub struct IssuedToken {
    pub secret: String,
    pub hash: String,
}

pub fn issue() -> Result<IssuedToken, AppError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(AppError::internal)?;
    let secret = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hex::encode(Sha256::digest(secret.as_bytes()));
    Ok(IssuedToken { secret, hash })
}

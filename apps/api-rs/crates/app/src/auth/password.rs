use argon2::password_hash::Error;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};

use super::error::AuthError;

pub async fn hash(password: String) -> Result<String, AuthError> {
    let hashed = tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
    })
    .await?;
    Ok(hashed?)
}

pub async fn verify(hash: String, password: String) -> Result<bool, AuthError> {
    let outcome = tokio::task::spawn_blocking(move || {
        Argon2::default().verify_password(password.as_bytes(), hash.as_str())
    })
    .await?;
    match outcome {
        Ok(()) => Ok(true),
        Err(Error::PasswordInvalid) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub fn unmatchable_hash() -> Result<String, Error> {
    Argon2::default()
        .hash_password(b"a hash no account owns, verified so an unknown email costs the same")
        .map(|hash| hash.to_string())
}

use argon2::{Argon2, PasswordHasher};

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

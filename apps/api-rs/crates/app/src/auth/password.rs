use argon2::{Argon2, PasswordHasher};

use crate::error::AppError;

pub async fn hash(password: String) -> Result<String, AppError> {
    let hashed = tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
    })
    .await
    .map_err(AppError::internal)?;
    hashed.map_err(AppError::internal)
}

use std::num::NonZeroUsize;
use std::sync::LazyLock;

use argon2::password_hash::Error;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use tokio::sync::Semaphore;

use super::error::CredentialError;

static HASHING_SLOTS: LazyLock<Semaphore> = LazyLock::new(|| {
    Semaphore::new(std::thread::available_parallelism().map_or(1, NonZeroUsize::get))
});

pub async fn hash(password: String) -> Result<String, CredentialError> {
    let hashed = on_a_hashing_slot(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
    })
    .await?;
    Ok(hashed?)
}

pub async fn verify(hash: String, password: String) -> Result<bool, CredentialError> {
    let outcome = on_a_hashing_slot(move || {
        Argon2::default().verify_password(password.as_bytes(), hash.as_str())
    })
    .await?;
    match outcome {
        Ok(()) => Ok(true),
        Err(Error::PasswordInvalid) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub const UNMATCHABLE_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$ggfuwG4ACktlFlCm+IYpRQ$85TQMLv3S0xBlNO3Mcz0dnha4o6lB3vUVFM9Xd7BzIQ";

async fn on_a_hashing_slot<Output: Send + 'static>(
    work: impl FnOnce() -> Output + Send + 'static,
) -> Result<Output, CredentialError> {
    let slot = HASHING_SLOTS.acquire().await?;
    let output = tokio::task::spawn_blocking(move || {
        let _held = slot;
        work()
    })
    .await?;
    Ok(output)
}

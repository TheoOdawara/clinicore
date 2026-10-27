use std::num::NonZeroUsize;
use std::sync::LazyLock;

use argon2::password_hash::Error;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use regex::Regex;
use tokio::sync::Semaphore;
use validator::ValidationError;

use super::error::CredentialError;

static HASHING_SLOTS: LazyLock<Semaphore> = LazyLock::new(|| {
    Semaphore::new(std::thread::available_parallelism().map_or(1, NonZeroUsize::get))
});

pub async fn hash(password: String) -> String {
    on_a_hashing_slot(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .expect("Argon2 with its default parameters hashes any password")
            .to_string()
    })
    .await
}

pub async fn verify(hash: String, password: String) -> Result<bool, CredentialError> {
    let outcome = on_a_hashing_slot(move || {
        Argon2::default().verify_password(password.as_bytes(), hash.as_str())
    })
    .await;
    match outcome {
        Ok(()) => Ok(true),
        Err(Error::PasswordInvalid) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub const UNMATCHABLE_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$ggfuwG4ACktlFlCm+IYpRQ$85TQMLv3S0xBlNO3Mcz0dnha4o6lB3vUVFM9Xd7BzIQ";

async fn on_a_hashing_slot<Output: Send + 'static>(
    work: impl FnOnce() -> Output + Send + 'static,
) -> Output {
    let slot = HASHING_SLOTS
        .acquire()
        .await
        .expect("the hashing semaphore is never closed");
    tokio::task::spawn_blocking(move || {
        let _held = slot;
        work()
    })
    .await
    .expect("hashing never panics and the runtime outlives the request")
}

pub fn strong_password(password: &str) -> Result<(), ValidationError> {
    static UPPERCASE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\p{Lu}").expect("a valid pattern"));
    static DIGIT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\p{Nd}").expect("a valid pattern"));
    static NEITHER_LETTER_NOR_DIGIT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[^\p{L}\p{Nd}]").expect("a valid pattern"));

    let is_strong = (8..=128).contains(&password.chars().count())
        && UPPERCASE.is_match(password)
        && DIGIT.is_match(password)
        && NEITHER_LETTER_NOR_DIGIT.is_match(password);
    if !is_strong {
        return Err(ValidationError::new("weak_password"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::strong_password;

    #[test]
    fn a_strong_password_has_8_to_128_characters_with_an_uppercase_a_digit_and_a_symbol() {
        let padded = |length: usize| format!("Aa#1{}", "a".repeat(length - 4));

        for strong in [padded(8), padded(128), "Ção#2026".to_string()] {
            assert!(strong_password(&strong).is_ok(), "{strong}");
        }
        for weak in [
            padded(7),
            padded(129),
            "sem_maiuscula#1".to_string(),
            "SEM_DIGITO#a".to_string(),
            "SemEspecial1".to_string(),
        ] {
            let error = strong_password(&weak).expect_err(&weak);
            assert_eq!(error.code, "weak_password", "{weak}");
        }
    }
}

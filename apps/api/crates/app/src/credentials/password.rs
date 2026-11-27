use std::sync::LazyLock;

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

use crate::credentials::secret::Secret;

const MIN_LENGTH: usize = 8;
const MAX_LENGTH: usize = 128;

pub fn meets_policy(password: &str) -> bool {
    let length = password.chars().count();
    (MIN_LENGTH..=MAX_LENGTH).contains(&length)
        && password.chars().any(char::is_uppercase)
        && password.chars().any(char::is_numeric)
        && password
            .chars()
            .any(|character| !character.is_alphanumeric())
}

static UNMATCHABLE: LazyLock<String> = LazyLock::new(|| hash_now(Secret::generate().as_str()));

pub fn prepare() {
    LazyLock::force(&UNMATCHABLE);
}

pub async fn hash(password: String) -> String {
    tokio::task::spawn_blocking(move || hash_now(&password))
        .await
        .expect("hashing a password never panics")
}

pub async fn verify(password: String, hash: Option<String>) -> bool {
    tokio::task::spawn_blocking(move || {
        let matches = Argon2::default()
            .verify_password(password.as_bytes(), hash.as_deref().unwrap_or(&UNMATCHABLE))
            .is_ok();
        matches && hash.is_some()
    })
    .await
    .expect("verifying a password never panics")
}

fn hash_now(password: &str) -> String {
    Argon2::default()
        .hash_password(password.as_bytes())
        .expect("argon2 with default parameters hashes any password")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_with_every_class_and_eight_characters_meets_the_policy() {
        assert!(meets_policy("Clinica#2026"));
        assert!(meets_policy("Abcdef1!"));
    }

    #[test]
    fn each_missing_class_fails_the_policy() {
        for password in ["clinica#2026", "Clinica#abcd", "Clinica2026", "Abcde1!"] {
            assert!(!meets_policy(password), "{password}");
        }
    }

    #[test]
    fn the_length_counts_characters_up_to_128() {
        let longest = format!("Ação#1{}", "a".repeat(122));
        let too_long = format!("{longest}a");

        assert_eq!(longest.chars().count(), 128);
        assert!(meets_policy(&longest));
        assert!(!meets_policy(&too_long));
    }

    #[tokio::test]
    async fn the_hash_is_argon2id_and_never_the_password() {
        let hashed = hash("Clinica#2026".to_owned()).await;

        assert!(hashed.starts_with("$argon2id$"), "{hashed}");
        assert!(!hashed.contains("Clinica#2026"));
    }

    #[tokio::test]
    async fn only_the_hashed_password_verifies() {
        let hashed = hash("Clinica#2026".to_owned()).await;

        assert!(verify("Clinica#2026".to_owned(), Some(hashed.clone())).await);
        assert!(!verify("Errada#2026".to_owned(), Some(hashed)).await);
        assert!(!verify("Clinica#2026".to_owned(), Some("not a hash".to_owned())).await);
    }

    #[tokio::test]
    async fn no_hash_never_verifies() {
        prepare();

        assert!(!verify("Clinica#2026".to_owned(), None).await);
    }
}

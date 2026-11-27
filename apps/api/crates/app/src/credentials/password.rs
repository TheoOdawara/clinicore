use argon2::Argon2;
use argon2::password_hash::PasswordHasher;

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

pub async fn hash(password: String) -> String {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .expect("argon2 with default parameters hashes any password")
            .to_string()
    })
    .await
    .expect("hashing a password never panics")
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
}

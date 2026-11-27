use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

const LENGTH: usize = 43;

pub struct Secret(String);

impl Secret {
    pub fn generate() -> Self {
        let bytes: [u8; 32] = rand::random();
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn parse(raw: &str) -> Option<Self> {
        let is_base64url = raw
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
        (raw.len() == LENGTH && is_base64url).then(|| Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn sha256(&self) -> Vec<u8> {
        Sha256::digest(self.0.as_bytes()).to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_secret_is_43_base64url_characters_and_parses_back() {
        let secret = Secret::generate();

        assert_eq!(secret.as_str().len(), 43);
        let parsed = Secret::parse(secret.as_str()).expect("a generated secret parses");
        assert_eq!(parsed.sha256(), secret.sha256());
    }

    #[test]
    fn two_secrets_differ() {
        assert_ne!(Secret::generate().as_str(), Secret::generate().as_str());
    }

    #[test]
    fn the_hash_is_the_sha256_of_the_text() {
        let secret = Secret::parse(&"a".repeat(43)).expect("43 letters parse");

        assert_eq!(
            secret.sha256(),
            Sha256::digest("a".repeat(43).as_bytes()).to_vec()
        );
        assert_eq!(secret.sha256().len(), 32);
    }

    #[test]
    fn a_wrong_length_or_alphabet_is_refused() {
        for raw in [
            "a".repeat(42),
            "a".repeat(44),
            format!("{}+", "a".repeat(42)),
            format!("{}=", "a".repeat(42)),
        ] {
            assert!(Secret::parse(&raw).is_none(), "{raw}");
        }
        assert!(Secret::parse(&format!("{}-_", "a".repeat(41))).is_some());
    }
}

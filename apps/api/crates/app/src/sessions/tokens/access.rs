use std::time::Duration;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::client::Client;
use crate::sessions::SessionId;
use crate::users::UserId;

pub const ACCESS_LIFETIME: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub sid: Uuid,
    pub cli: Client,
    pub iat: u64,
    pub exp: u64,
}

pub fn now() -> u64 {
    jsonwebtoken::get_current_timestamp()
}

pub fn issue(
    secret: &str,
    user_id: UserId,
    session_id: SessionId,
    client: Client,
    now: u64,
) -> String {
    let claims = Claims {
        sub: user_id.0,
        sid: session_id.0,
        cli: client,
        iat: now,
        exp: now + ACCESS_LIFETIME.as_secs(),
    };
    let key = EncodingKey::from_secret(secret.as_bytes())
        .expect("the boot refuses a JWT_SECRET shorter than 32 characters");
    jsonwebtoken::encode(&Header::new(Algorithm::HS256), &claims, &key)
        .expect("HS256 signs any claims")
}

pub fn verify(secret: &str, token: &str) -> Option<Claims> {
    let key = DecodingKey::from_secret(secret.as_bytes())
        .expect("the boot refuses a JWT_SECRET shorter than 32 characters");
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    jsonwebtoken::decode::<Claims>(token, &key, &validation)
        .ok()
        .map(|data| data.claims)
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    use super::*;

    const SECRET: &str = "a-test-secret-with-at-least-32-characters";

    fn token_at(now: u64) -> String {
        issue(
            SECRET,
            UserId(Uuid::from_u128(1)),
            SessionId(Uuid::from_u128(2)),
            Client::Mobile,
            now,
        )
    }

    #[test]
    fn an_issued_token_is_hs256_and_verifies_with_its_claims() {
        let issued_at = now();
        let token = token_at(issued_at);
        let header = token.split('.').next().expect("the token has a header");

        assert_eq!(
            URL_SAFE_NO_PAD.decode(header).expect("base64url"),
            br#"{"typ":"JWT","alg":"HS256"}"#
        );
        assert_eq!(
            verify(SECRET, &token),
            Some(Claims {
                sub: Uuid::from_u128(1),
                sid: Uuid::from_u128(2),
                cli: Client::Mobile,
                iat: issued_at,
                exp: issued_at + 900,
            })
        );
    }

    #[test]
    fn an_expired_token_another_secret_or_another_algorithm_is_refused() {
        let token = token_at(now());
        let (_, rest) = token.split_once('.').expect("the token has a header");
        let none_header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#);

        assert_eq!(verify(SECRET, &token_at(now() - 901)), None);
        assert_eq!(
            verify("another-secret-with-at-least-32-characters", &token),
            None
        );
        assert_eq!(verify(SECRET, &format!("{none_header}.{rest}")), None);
        assert_eq!(verify(SECRET, "garbage"), None);
    }
}

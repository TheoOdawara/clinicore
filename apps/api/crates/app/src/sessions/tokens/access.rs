use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

use crate::http::client::Client;
use crate::sessions::SessionId;
use crate::users::UserId;

pub const ACCESS_LIFETIME: Duration = Duration::from_secs(15 * 60);
const HEADER: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";

type Signer = Hmac<Sha256>;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub sid: Uuid,
    pub cli: Client,
    pub iat: u64,
    pub exp: u64,
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is past 1970")
        .as_secs()
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
    let payload = serde_json::to_vec(&claims).expect("the claims serialize");
    let signing_input = format!("{HEADER}.{}", URL_SAFE_NO_PAD.encode(payload));
    let signature = signer(secret)
        .chain_update(signing_input.as_bytes())
        .finalize()
        .into_bytes();
    format!("{signing_input}.{}", URL_SAFE_NO_PAD.encode(signature))
}

pub fn verify(secret: &str, token: &str, now: u64) -> Option<Claims> {
    let (signing_input, signature) = token.rsplit_once('.')?;
    let (header, payload) = signing_input.split_once('.')?;
    if header != HEADER {
        return None;
    }
    let signature = URL_SAFE_NO_PAD.decode(signature).ok()?;
    signer(secret)
        .chain_update(signing_input.as_bytes())
        .verify_slice(&signature)
        .ok()?;
    let payload = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let claims: Claims = serde_json::from_slice(&payload).ok()?;
    (claims.exp > now).then_some(claims)
}

fn signer(secret: &str) -> Signer {
    Signer::new_from_slice(secret.as_bytes()).expect("hmac takes a key of any length")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "a-test-secret-with-at-least-32-characters";
    const NOW: u64 = 1_800_000_000;

    fn token() -> String {
        issue(
            SECRET,
            UserId(Uuid::from_u128(1)),
            SessionId(Uuid::from_u128(2)),
            Client::Mobile,
            NOW,
        )
    }

    #[test]
    fn the_header_is_hs256() {
        let header = URL_SAFE_NO_PAD
            .decode(HEADER)
            .expect("the header is base64url");

        assert_eq!(header, br#"{"alg":"HS256","typ":"JWT"}"#);
    }

    #[test]
    fn an_issued_token_verifies_with_its_claims_for_900_seconds() {
        let claims = verify(SECRET, &token(), NOW + 899).expect("the token is valid");

        assert_eq!(
            claims,
            Claims {
                sub: Uuid::from_u128(1),
                sid: Uuid::from_u128(2),
                cli: Client::Mobile,
                iat: NOW,
                exp: NOW + 900,
            }
        );
        assert_eq!(verify(SECRET, &token(), NOW + 900), None);
    }

    #[test]
    fn another_secret_or_a_changed_part_is_refused() {
        let token = token();
        let (signing_input, signature) = token.rsplit_once('.').expect("three parts");
        let (_, payload) = signing_input.split_once('.').expect("three parts");
        let none_header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#);
        let forged_payload = URL_SAFE_NO_PAD.encode(
            br#"{"sub":"00000000-0000-0000-0000-000000000009","sid":"00000000-0000-0000-0000-000000000002","cli":"mobile","iat":1800000000,"exp":1800000900}"#,
        );

        assert_eq!(
            verify("another-secret-with-at-least-32-characters", &token, NOW),
            None
        );
        for forged in [
            format!("{none_header}.{payload}.{signature}"),
            format!("{none_header}.{payload}."),
            format!("{HEADER}.{forged_payload}.{signature}"),
            format!("{signing_input}.not-base64!"),
            signing_input.to_owned(),
            "garbage".to_owned(),
        ] {
            assert_eq!(verify(SECRET, &forged, NOW), None, "{forged}");
        }
    }

    fn signed(payload: &str) -> String {
        let signing_input = format!("{HEADER}.{payload}");
        let signature = signer(SECRET)
            .chain_update(signing_input.as_bytes())
            .finalize()
            .into_bytes();
        format!("{signing_input}.{}", URL_SAFE_NO_PAD.encode(signature))
    }

    #[test]
    fn a_signed_payload_that_is_not_the_claims_is_refused() {
        assert_eq!(
            verify(SECRET, &signed(&URL_SAFE_NO_PAD.encode(b"{}")), NOW),
            None
        );
        assert_eq!(verify(SECRET, &signed("!"), NOW), None);
    }
}

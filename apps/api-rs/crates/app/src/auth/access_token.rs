use std::time::Duration;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::client::SessionClient;

pub const LIFETIME: Duration = Duration::from_secs(900);

#[derive(Clone)]
pub struct AccessKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
}

impl AccessKeys {
    pub fn new(secret: &str) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            validation,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub sid: Uuid,
    pub cli: SessionClient,
    exp: u64,
}

pub fn sign(
    keys: &AccessKeys,
    user_id: Uuid,
    session_id: Uuid,
    client: SessionClient,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: user_id,
        sid: session_id,
        cli: client,
        exp: jsonwebtoken::get_current_timestamp() + LIFETIME.as_secs(),
    };
    jsonwebtoken::encode(&Header::default(), &claims, &keys.encoding)
}

pub fn verify(keys: &AccessKeys, token: &str) -> Option<Claims> {
    jsonwebtoken::decode(token, &keys.decoding, &keys.validation)
        .ok()
        .map(|data| data.claims)
}

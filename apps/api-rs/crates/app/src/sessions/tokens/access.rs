use std::time::Duration;

use clinicore_core::redis::{Redis, RedisError};
use jsonwebtoken::errors::{Error, ErrorKind};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::refresh;
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
    #[serde(rename = "sub")]
    pub user_id: Uuid,
    #[serde(rename = "sid")]
    pub session_id: Uuid,
    #[serde(rename = "cli")]
    pub client: SessionClient,
    exp: u64,
}

pub fn sign(
    keys: &AccessKeys,
    user_id: Uuid,
    session_id: Uuid,
    client: SessionClient,
) -> Result<String, Error> {
    let claims = Claims {
        user_id,
        session_id,
        client,
        exp: jsonwebtoken::get_current_timestamp() + LIFETIME.as_secs(),
    };
    jsonwebtoken::encode(&Header::default(), &claims, &keys.encoding)
}

pub fn verify(keys: &AccessKeys, token: &str) -> Result<Option<Claims>, Error> {
    let error = match jsonwebtoken::decode(token, &keys.decoding, &keys.validation) {
        Ok(data) => return Ok(Some(data.claims)),
        Err(error) => error,
    };
    match error.kind() {
        ErrorKind::InvalidToken
        | ErrorKind::InvalidSignature
        | ErrorKind::InvalidAlgorithmName
        | ErrorKind::UnsupportedAlgorithm
        | ErrorKind::MissingRequiredClaim(_)
        | ErrorKind::InvalidClaimFormat(_)
        | ErrorKind::ExpiredSignature
        | ErrorKind::InvalidIssuer
        | ErrorKind::InvalidAudience
        | ErrorKind::InvalidSubject
        | ErrorKind::ImmatureSignature
        | ErrorKind::InvalidAlgorithm
        | ErrorKind::MissingAlgorithm
        | ErrorKind::Base64(_)
        | ErrorKind::Json(_)
        | ErrorKind::Utf8(_) => Ok(None),
        _ => Err(error),
    }
}

pub async fn revoke(redis: &Redis, session_ids: &[Uuid]) -> Result<(), RedisError> {
    let keys: Vec<String> = session_ids.iter().copied().map(revoked_key).collect();
    redis.set_expiring(&keys, refresh::longest_lifetime()).await
}

pub async fn is_revoked(redis: &Redis, session_id: Uuid) -> Result<bool, RedisError> {
    redis.exists(&revoked_key(session_id)).await
}

fn revoked_key(session_id: Uuid) -> String {
    format!("auth:revoked:{session_id}")
}

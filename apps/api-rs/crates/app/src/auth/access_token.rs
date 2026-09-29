use std::time::Duration;

use jsonwebtoken::errors::{Error, ErrorKind};
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
) -> Result<String, Error> {
    let claims = Claims {
        sub: user_id,
        sid: session_id,
        cli: client,
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

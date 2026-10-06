use clinicore_core::mail::MailError;
use clinicore_core::redis::RedisError;

use crate::credentials::error::CredentialError;
use crate::http::error::AppError;
use crate::http::rate_limit::RateLimitError;

#[derive(Debug, thiserror::Error)]
pub enum PasswordResetError {
    #[error("invalid token")]
    InvalidToken,
    #[error(transparent)]
    RateLimit(#[from] RateLimitError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error(transparent)]
    Mail(#[from] MailError),
    #[error(transparent)]
    Credential(#[from] CredentialError),
}

impl From<PasswordResetError> for AppError {
    fn from(error: PasswordResetError) -> Self {
        match error {
            PasswordResetError::InvalidToken => Self::InvalidToken,
            PasswordResetError::RateLimit(error) => error.into(),
            PasswordResetError::Database(error) => Self::Database(error),
            PasswordResetError::Redis(error) => Self::Unavailable(error),
            other => Self::Internal(Box::new(other)),
        }
    }
}

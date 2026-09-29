use clinicore_core::redis::RedisError;

use crate::credentials::error::CredentialError;
use crate::email_verifications::error::EmailVerificationError;
use crate::http::error::AppError;
use crate::http::rate_limit::RateLimitError;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("email not verified")]
    EmailNotVerified,
    #[error("invalid session")]
    InvalidSession,
    #[error("session reused")]
    SessionReused,
    #[error(transparent)]
    RateLimit(#[from] RateLimitError),
    #[error(transparent)]
    EmailVerification(#[from] EmailVerificationError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error(transparent)]
    Credential(#[from] CredentialError),
    #[error(transparent)]
    AccessToken(#[from] jsonwebtoken::errors::Error),
}

impl From<SessionError> for AppError {
    fn from(error: SessionError) -> Self {
        match error {
            SessionError::InvalidCredentials => Self::InvalidCredentials,
            SessionError::EmailNotVerified => Self::EmailNotVerified,
            SessionError::InvalidSession => Self::InvalidSession,
            SessionError::SessionReused => Self::SessionReused,
            SessionError::RateLimit(error) => error.into(),
            SessionError::EmailVerification(error) => error.into(),
            SessionError::Database(error) => Self::Database(error),
            SessionError::Redis(error) => Self::Unavailable(error),
            other => Self::Internal(Box::new(other)),
        }
    }
}

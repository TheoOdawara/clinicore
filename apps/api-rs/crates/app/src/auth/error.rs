use clinicore_core::mail::MailError;
use clinicore_core::redis::RedisError;

use crate::http::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("email not verified")]
    EmailNotVerified,
    #[error("invalid session")]
    InvalidSession,
    #[error("session reused")]
    SessionReused,
    #[error("invalid token")]
    InvalidToken,
    #[error("token expired")]
    TokenExpired,
    #[error("rate limited")]
    RateLimited,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error(transparent)]
    Mail(#[from] MailError),
    #[error(transparent)]
    PasswordHash(#[from] argon2::password_hash::Error),
    #[error(transparent)]
    AccessToken(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    Random(#[from] getrandom::Error),
    #[error(transparent)]
    Task(#[from] tokio::task::JoinError),
    #[error(transparent)]
    HashingSlot(#[from] tokio::sync::AcquireError),
}

impl From<AuthError> for AppError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::InvalidCredentials => Self::InvalidCredentials,
            AuthError::EmailNotVerified => Self::EmailNotVerified,
            AuthError::InvalidSession => Self::InvalidSession,
            AuthError::SessionReused => Self::SessionReused,
            AuthError::InvalidToken => Self::InvalidToken,
            AuthError::TokenExpired => Self::TokenExpired,
            AuthError::RateLimited => Self::RateLimited,
            AuthError::Database(error) => Self::Database(error),
            AuthError::Redis(error) => Self::Unavailable(error),
            other => Self::Internal(Box::new(other)),
        }
    }
}

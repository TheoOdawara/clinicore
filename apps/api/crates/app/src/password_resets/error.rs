use clinicore_core::redis::RedisError;

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
}

impl From<PasswordResetError> for AppError {
    fn from(error: PasswordResetError) -> Self {
        match error {
            PasswordResetError::InvalidToken => Self::InvalidToken,
            PasswordResetError::RateLimit(error) => error.into(),
            PasswordResetError::Database(error) => Self::Database(error),
            PasswordResetError::Redis(error) => Self::Unavailable(error),
        }
    }
}

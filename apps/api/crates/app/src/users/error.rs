use clinicore_core::redis::RedisError;

use crate::credentials::error::CredentialError;
use crate::http::error::AppError;
use crate::http::rate_limit::RateLimitError;

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("invalid password")]
    InvalidPassword,
    #[error(transparent)]
    RateLimit(#[from] RateLimitError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error(transparent)]
    Template(#[from] askama::Error),
    #[error(transparent)]
    Credential(#[from] CredentialError),
}

impl From<UserError> for AppError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::InvalidPassword => Self::InvalidPassword,
            UserError::RateLimit(error) => error.into(),
            UserError::Database(error) => Self::Database(error),
            UserError::Redis(error) => Self::Unavailable(error),
            other => Self::Internal(Box::new(other)),
        }
    }
}

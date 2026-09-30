use clinicore_core::mail::MailError;

use crate::credentials::error::CredentialError;
use crate::http::error::AppError;
use crate::http::rate_limit::RateLimitError;

#[derive(Debug, thiserror::Error)]
pub enum EmailVerificationError {
    #[error("invalid token")]
    InvalidToken,
    #[error("token expired")]
    TokenExpired,
    #[error(transparent)]
    RateLimit(#[from] RateLimitError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Mail(#[from] MailError),
    #[error(transparent)]
    Credential(#[from] CredentialError),
}

impl From<EmailVerificationError> for AppError {
    fn from(error: EmailVerificationError) -> Self {
        match error {
            EmailVerificationError::InvalidToken => Self::InvalidToken,
            EmailVerificationError::TokenExpired => Self::TokenExpired,
            EmailVerificationError::RateLimit(error) => error.into(),
            EmailVerificationError::Database(error) => Self::Database(error),
            other => Self::Internal(Box::new(other)),
        }
    }
}

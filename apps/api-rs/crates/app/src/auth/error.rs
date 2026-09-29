use clinicore_core::mail::MailError;

use crate::http::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Mail(#[from] MailError),
    #[error(transparent)]
    PasswordHash(#[from] argon2::password_hash::Error),
    #[error(transparent)]
    Random(#[from] getrandom::Error),
    #[error(transparent)]
    Task(#[from] tokio::task::JoinError),
}

impl From<AuthError> for AppError {
    fn from(error: AuthError) -> Self {
        Self::Internal(Box::new(error))
    }
}

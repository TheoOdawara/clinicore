use clinicore_core::mail::MailError;

use crate::credentials::error::CredentialError;
use crate::http::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Mail(#[from] MailError),
    #[error(transparent)]
    Credential(#[from] CredentialError),
}

impl From<UserError> for AppError {
    fn from(error: UserError) -> Self {
        match error {
            UserError::Database(error) => Self::Database(error),
            other => Self::Internal(Box::new(other)),
        }
    }
}

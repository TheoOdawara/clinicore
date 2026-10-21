use super::google::GoogleError;
use crate::http::error::AppError;
use crate::http::rate_limit::RateLimitError;
use crate::sessions::error::SessionError;

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("the state does not match the one issued")]
    InvalidState,
    #[error("the provider has not verified the email")]
    UnverifiedProviderEmail,
    #[error("the provider returned no authorization code")]
    AuthorizationDenied,
    #[error("the user is linked to another account of the provider")]
    LinkedToAnotherAccount,
    #[error(transparent)]
    Provider(#[from] GoogleError),
    #[error(transparent)]
    RateLimit(#[from] RateLimitError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Session(#[from] SessionError),
}

impl OAuthError {
    pub fn redirect_code(self) -> Result<&'static str, AppError> {
        match self {
            Self::InvalidState => Ok("INVALID_STATE"),
            Self::UnverifiedProviderEmail => Ok("UNVERIFIED_PROVIDER_EMAIL"),
            Self::AuthorizationDenied | Self::LinkedToAnotherAccount | Self::Provider(_) => {
                Ok("PROVIDER_ERROR")
            }
            Self::RateLimit(error) => Err(error.into()),
            Self::Database(error) => Err(AppError::Database(error)),
            Self::Session(error) => Err(error.into()),
        }
    }
}

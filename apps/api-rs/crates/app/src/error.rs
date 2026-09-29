use std::error::Error;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorCode {
    ValidationFailed,
    InvalidOrigin,
    RateLimited,
    ServiceUnavailable,
}

#[derive(Debug)]
pub enum AppError {
    Business(ErrorCode),
    Internal(Box<dyn Error + Send + Sync>),
}

impl AppError {
    pub fn internal(cause: impl Error + Send + Sync + 'static) -> Self {
        Self::Internal(Box::new(cause))
    }
}

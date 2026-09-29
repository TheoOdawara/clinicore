use std::error::Error;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorCode {
    InvalidOrigin,
}

#[derive(Debug)]
pub enum AppError {
    Business(ErrorCode),
    Internal(Box<dyn Error + Send + Sync>),
}

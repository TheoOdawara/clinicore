use std::any::Any;
use std::error::Error;

use axum::Json;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use clinicore_core::redis::RedisError;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation failed")]
    Validation(Vec<FieldError>),
    #[error("Invalid client")]
    InvalidClient,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Invalid email or password")]
    InvalidCredentials,
    #[error("Invalid session")]
    InvalidSession,
    #[error("Refresh token reuse detected")]
    SessionReused,
    #[error("Email not verified")]
    EmailNotVerified,
    #[error("Invalid origin")]
    InvalidOrigin,
    #[error("Too many requests")]
    RateLimited,
    #[error("Service temporarily unavailable")]
    Unavailable(#[from] RedisError),
    #[error("{0}")]
    Rejected(StatusCode),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    Internal(Box<dyn Error + Send + Sync>),
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FieldError {
    pub pointer: String,
    pub code: String,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct Problem {
    #[serde(rename = "type")]
    problem_type: String,
    title: String,
    status: u16,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<FieldError>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let title = self.to_string();
        let (status, slug, errors) = match self {
            Self::Validation(errors) => (StatusCode::BAD_REQUEST, "validation-failed", errors),
            Self::InvalidClient => (StatusCode::BAD_REQUEST, "invalid-client", Vec::new()),
            Self::InvalidToken => (StatusCode::BAD_REQUEST, "invalid-token", Vec::new()),
            Self::TokenExpired => (StatusCode::BAD_REQUEST, "token-expired", Vec::new()),
            Self::InvalidPassword => (StatusCode::BAD_REQUEST, "invalid-password", Vec::new()),
            Self::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "invalid-credentials", Vec::new())
            }
            Self::InvalidSession => (StatusCode::UNAUTHORIZED, "invalid-session", Vec::new()),
            Self::SessionReused => (StatusCode::UNAUTHORIZED, "session-reused", Vec::new()),
            Self::EmailNotVerified => (StatusCode::FORBIDDEN, "email-not-verified", Vec::new()),
            Self::InvalidOrigin => (StatusCode::FORBIDDEN, "invalid-origin", Vec::new()),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate-limited", Vec::new()),
            Self::Unavailable(cause) => {
                tracing::error!(cause = %cause, "a dependency is unavailable");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "service-unavailable",
                    Vec::new(),
                )
            }
            Self::Rejected(status) => return blank(status),
            Self::Database(_) | Self::Internal(_) => {
                tracing::error!(cause = %title, "unhandled error");
                return blank(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        problem(
            status,
            format!("tag:clinicore.com.br,2026:{slug}"),
            title,
            errors,
        )
    }
}

fn blank(status: StatusCode) -> Response {
    let title = status.canonical_reason().unwrap_or("Unknown Error").into();
    problem(status, "about:blank".into(), title, Vec::new())
}

fn problem(
    status: StatusCode,
    problem_type: String,
    title: String,
    errors: Vec<FieldError>,
) -> Response {
    let body = Problem {
        problem_type,
        title,
        status: status.as_u16(),
        errors,
    };
    (
        status,
        [(CONTENT_TYPE, "application/problem+json")],
        Json(body),
    )
        .into_response()
}

pub(crate) fn panic_response(payload: Box<dyn Any + Send + 'static>) -> Response {
    let cause = payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|message| message.to_string())
        })
        .unwrap_or_else(|| "a panic with no message".to_string());
    tracing::error!(cause = %cause, "unhandled panic");
    blank(StatusCode::INTERNAL_SERVER_ERROR)
}

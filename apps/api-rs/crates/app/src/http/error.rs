use std::any::Any;
use std::error::Error;

use axum::Json;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use clinicore_core::redis::RedisError;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation failed")]
    Validation(Vec<FieldError>),
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

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub pointer: String,
    pub code: String,
}

#[derive(Serialize)]
struct Problem {
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

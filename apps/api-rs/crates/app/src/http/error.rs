use std::any::Any;
use std::error::Error;

use axum::Json;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
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
    Unavailable,
    #[error("{0}")]
    Rejected(StatusCode),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    Internal(Box<dyn Error + Send + Sync>),
}

impl AppError {
    pub fn internal(cause: impl Error + Send + Sync + 'static) -> Self {
        Self::Internal(Box::new(cause))
    }
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
        let message = self.to_string();
        match self {
            Self::Validation(errors) => coded(
                StatusCode::BAD_REQUEST,
                "validation-failed",
                message,
                errors,
            ),
            Self::InvalidOrigin => {
                coded(StatusCode::FORBIDDEN, "invalid-origin", message, Vec::new())
            }
            Self::RateLimited => coded(
                StatusCode::TOO_MANY_REQUESTS,
                "rate-limited",
                message,
                Vec::new(),
            ),
            Self::Unavailable => coded(
                StatusCode::SERVICE_UNAVAILABLE,
                "service-unavailable",
                message,
                Vec::new(),
            ),
            Self::Rejected(status) => blank(status),
            Self::Database(_) | Self::Internal(_) => {
                tracing::error!(cause = %message, "unhandled error");
                blank(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

fn coded(status: StatusCode, slug: &str, title: String, errors: Vec<FieldError>) -> Response {
    problem(
        status,
        Problem {
            problem_type: format!("tag:clinicore.com.br,2026:{slug}"),
            title,
            status: status.as_u16(),
            errors,
        },
    )
}

fn blank(status: StatusCode) -> Response {
    let title = status.canonical_reason().unwrap_or("Unknown Error");
    problem(
        status,
        Problem {
            problem_type: "about:blank".to_string(),
            title: title.to_string(),
            status: status.as_u16(),
            errors: Vec::new(),
        },
    )
}

fn problem(status: StatusCode, body: Problem) -> Response {
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

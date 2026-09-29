use std::any::Any;

use api_app::error::{AppError, ErrorCode};
use axum::Json;
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub struct ApiError(pub AppError);

#[derive(Serialize)]
pub struct FieldError {
    pub pointer: String,
    pub code: String,
}

#[derive(Serialize)]
struct Problem {
    #[serde(rename = "type")]
    problem_type: String,
    title: &'static str,
    status: u16,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<FieldError>,
}

fn problem(
    status: StatusCode,
    problem_type: String,
    title: &'static str,
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

pub fn blank_problem(status: StatusCode) -> Response {
    let title = status.canonical_reason().unwrap_or("Unknown Error");
    problem(status, "about:blank".to_string(), title, Vec::new())
}

pub fn business_problem(code: ErrorCode, errors: Vec<FieldError>) -> Response {
    let (status, slug, title) = catalog(code);
    problem(
        status,
        format!("tag:clinicore.com.br,2026:{slug}"),
        title,
        errors,
    )
}

fn catalog(code: ErrorCode) -> (StatusCode, &'static str, &'static str) {
    match code {
        ErrorCode::ValidationFailed => (
            StatusCode::BAD_REQUEST,
            "validation-failed",
            "Validation failed",
        ),
        ErrorCode::InvalidOrigin => (StatusCode::FORBIDDEN, "invalid-origin", "Invalid origin"),
        ErrorCode::RateLimited => (
            StatusCode::TOO_MANY_REQUESTS,
            "rate-limited",
            "Too many requests",
        ),
        ErrorCode::ServiceUnavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "service-unavailable",
            "Service temporarily unavailable",
        ),
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self.0 {
            AppError::Business(code) => business_problem(code, Vec::new()),
            AppError::Internal(cause) => {
                tracing::error!(cause = %cause, "unhandled error");
                blank_problem(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
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
    blank_problem(StatusCode::INTERNAL_SERVER_ERROR)
}

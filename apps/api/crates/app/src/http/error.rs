use std::any::Any;

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

const PROBLEM_JSON: &str = "application/problem+json";
const ABOUT_BLANK: &str = "about:blank";
const PROBLEM_TYPE_PREFIX: &str = "tag:clinicore.com.br,2026:";

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("no route matches the path")]
    NotFound,
    #[error("the route does not accept the method")]
    MethodNotAllowed,
    #[error("the request failed inside the server")]
    Internal,
    #[error("the Clinicore-Client header names no known client")]
    InvalidClient,
    #[error("the origin is not allowed to change state")]
    InvalidOrigin,
    #[error("the body breaks the contract")]
    Validation(Vec<FieldError>),
    #[error("the body is not a JSON object")]
    MalformedBody,
    #[error("the body is larger than the limit")]
    PayloadTooLarge,
    #[error("the body is not declared as JSON")]
    UnsupportedMediaType,
    #[error("the limit of the route was reached")]
    RateLimited { retry_after: u64 },
    #[error("a service the request needs is unavailable")]
    ServiceUnavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
pub struct FieldError {
    pub pointer: String,
    pub code: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldError>>,
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidClient => StatusCode::BAD_REQUEST,
            Self::InvalidOrigin => StatusCode::FORBIDDEN,
            Self::Validation(_) | Self::MalformedBody => StatusCode::BAD_REQUEST,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn slug_and_title(&self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::NotFound
            | Self::MethodNotAllowed
            | Self::Internal
            | Self::MalformedBody
            | Self::PayloadTooLarge
            | Self::UnsupportedMediaType => None,
            Self::InvalidClient => Some(("invalid-client", "Invalid client")),
            Self::InvalidOrigin => Some(("invalid-origin", "Invalid origin")),
            Self::Validation(_) => Some(("validation-failed", "Validation failed")),
            Self::RateLimited { .. } => Some(("rate-limited", "Too many requests")),
            Self::ServiceUnavailable => {
                Some(("service-unavailable", "Service temporarily unavailable"))
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let retry_after = match self {
            Self::RateLimited { retry_after } => Some(retry_after),
            _ => None,
        };
        let (problem_type, title) = match self.slug_and_title() {
            Some((slug, title)) => (format!("{PROBLEM_TYPE_PREFIX}{slug}"), title),
            None => (
                ABOUT_BLANK.to_owned(),
                status
                    .canonical_reason()
                    .expect("every status the API answers has a reason phrase"),
            ),
        };
        let errors = match self {
            Self::Validation(errors) => Some(errors),
            _ => None,
        };
        let problem = Problem {
            problem_type,
            title: title.to_owned(),
            status: status.as_u16(),
            errors,
        };
        let mut response = (status, Json(problem)).into_response();
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static(PROBLEM_JSON));
        if let Some(seconds) = retry_after {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(seconds));
        }
        response
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "the database failed the request");
        Self::Internal
    }
}

pub async fn not_found() -> AppError {
    AppError::NotFound
}

pub async fn method_not_allowed() -> AppError {
    AppError::MethodNotAllowed
}

pub fn panic_response(payload: Box<dyn Any + Send + 'static>) -> Response {
    let message = payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_default();
    tracing::error!(panic = %message, "a handler panicked");
    AppError::Internal.into_response()
}

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
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidClient => StatusCode::BAD_REQUEST,
            Self::InvalidOrigin => StatusCode::FORBIDDEN,
        }
    }

    fn slug_and_title(&self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::NotFound | Self::MethodNotAllowed | Self::Internal => None,
            Self::InvalidClient => Some(("invalid-client", "Invalid client")),
            Self::InvalidOrigin => Some(("invalid-origin", "Invalid origin")),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let (problem_type, title) = match self.slug_and_title() {
            Some((slug, title)) => (format!("{PROBLEM_TYPE_PREFIX}{slug}"), title),
            None => (
                ABOUT_BLANK.to_owned(),
                status
                    .canonical_reason()
                    .expect("every status the API answers has a reason phrase"),
            ),
        };
        let problem = Problem {
            problem_type,
            title: title.to_owned(),
            status: status.as_u16(),
        };
        let mut response = (status, Json(problem)).into_response();
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static(PROBLEM_JSON));
        response
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

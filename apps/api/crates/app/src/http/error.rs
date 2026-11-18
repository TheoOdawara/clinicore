use std::any::Any;

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

const PROBLEM_JSON: &str = "application/problem+json";
const ABOUT_BLANK: &str = "about:blank";

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("no route matches the path")]
    NotFound,
    #[error("the route does not accept the method")]
    MethodNotAllowed,
    #[error("the request failed inside the server")]
    Internal,
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
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let problem = Problem {
            problem_type: ABOUT_BLANK.to_owned(),
            title: status
                .canonical_reason()
                .expect("every status the API answers has a reason phrase")
                .to_owned(),
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

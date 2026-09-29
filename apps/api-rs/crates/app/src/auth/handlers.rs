use axum::extract::State;
use axum::http::StatusCode;

use super::requests::{EmailVerificationRequest, SignUpRequest};
use super::service;
use crate::AppState;
use crate::http::error::AppError;
use crate::http::validation::ValidJson;

#[utoipa::path(
    post,
    path = "/users",
    tag = "Auth",
    summary = "Register an email and password account",
    description = "Always answers 202 with an empty body, whether the email is new or already registered.",
    request_body = SignUpRequest,
    responses((status = 202, description = "Accepted, with no body"))
)]
pub async fn sign_up(
    State(state): State<AppState>,
    ValidJson(body): ValidJson<SignUpRequest>,
) -> Result<StatusCode, AppError> {
    service::sign_up(&state, &body.name, &body.email, &body.password).await?;
    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    post,
    path = "/email-verifications",
    tag = "Auth",
    summary = "Send a new email verification link",
    description = "Always answers 202 with an empty body. At most one link per address every 60 seconds and five every 24 hours.",
    request_body = EmailVerificationRequest,
    responses((status = 202, description = "Accepted, with no body"))
)]
pub async fn request_email_verification(
    State(state): State<AppState>,
    ValidJson(body): ValidJson<EmailVerificationRequest>,
) -> Result<StatusCode, AppError> {
    service::request_email_verification(&state, &body.email).await?;
    Ok(StatusCode::ACCEPTED)
}

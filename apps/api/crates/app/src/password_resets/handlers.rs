use axum::extract::State;
use axum::http::StatusCode;

use super::requests::{PasswordResetConfirmationRequest, PasswordResetRequest};
use super::service;
use crate::AppState;
use crate::email_dispatches::dispatch_limits_text;
use crate::http::client::ClientAddress;
use crate::http::error::{AppError, Problem};
use crate::http::validation::ValidJson;

#[utoipa::path(
    post,
    path = "/password-resets",
    tag = "Auth",
    summary = "Send a password reset link",
    description = format!("Always answers 202 with an empty body, whether the email has an account or not. {}", dispatch_limits_text()),
    request_body = PasswordResetRequest,
    responses((status = 202, description = "Accepted, with no body"))
)]
pub async fn request_password_reset(
    State(state): State<AppState>,
    ValidJson(body): ValidJson<PasswordResetRequest>,
) -> Result<StatusCode, AppError> {
    service::request_password_reset(&state, &body.email).await?;
    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    post,
    path = "/password-resets/confirmation",
    tag = "Auth",
    summary = "Set a new password with the reset link",
    description = "Revokes every session of the user at once. An account that only signed in with Google gains the password sign-in.",
    request_body = PasswordResetConfirmationRequest,
    responses(
        (status = 204, description = "Password set"),
        (status = 400, description = "invalid-token; or invalid-client, or validation-failed", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn confirm_password_reset(
    State(state): State<AppState>,
    client: ClientAddress,
    ValidJson(body): ValidJson<PasswordResetConfirmationRequest>,
) -> Result<StatusCode, AppError> {
    service::confirm_password_reset(&state, client, &body.token, &body.new_password).await?;
    Ok(StatusCode::NO_CONTENT)
}

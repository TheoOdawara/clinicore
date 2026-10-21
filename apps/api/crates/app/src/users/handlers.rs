use axum::extract::State;
use axum::http::StatusCode;

use super::requests::{PasswordChangeRequest, SignUpRequest};
use super::service;
use crate::AppState;
use crate::http::error::{AppError, Problem};
use crate::http::validation::ValidJson;
use crate::sessions::extractors::CurrentSession;

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
    put,
    path = "/users/me/password",
    tag = "Auth",
    summary = "Change the password of the signed-in user",
    description = "Keeps the session that made the request and revokes every other one at once.",
    request_body = PasswordChangeRequest,
    responses(
        (status = 204, description = "Password changed"),
        (status = 400, description = "invalid-password; or invalid-client, or validation-failed", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "invalid-session", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn change_password(
    State(state): State<AppState>,
    session: CurrentSession,
    ValidJson(body): ValidJson<PasswordChangeRequest>,
) -> Result<StatusCode, AppError> {
    service::change_password(&state, &session, &body.current_password, &body.new_password).await?;
    Ok(StatusCode::NO_CONTENT)
}

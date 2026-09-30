use axum::extract::State;
use axum::http::StatusCode;

use super::requests::{EmailConfirmationRequest, EmailVerificationRequest};
use super::service;
use crate::AppState;
use crate::http::client::ClientAddress;
use crate::http::error::{AppError, Problem};
use crate::http::validation::ValidJson;

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

#[utoipa::path(
    post,
    path = "/email-verifications/confirmation",
    tag = "Auth",
    summary = "Confirm the email",
    description = "Marks the email as verified and opens no session: the owner signs in afterwards with the password they chose.",
    request_body = EmailConfirmationRequest,
    responses(
        (status = 204, description = "Confirmed"),
        (status = 400, description = "invalid-token or token-expired; or invalid-client, or validation-failed", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn confirm_email(
    State(state): State<AppState>,
    client: ClientAddress,
    ValidJson(body): ValidJson<EmailConfirmationRequest>,
) -> Result<StatusCode, AppError> {
    service::confirm_email(&state, client, &body.token).await?;
    Ok(StatusCode::NO_CONTENT)
}

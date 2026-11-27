use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;

use crate::AppState;
use crate::email_verifications::requests::{
    EmailVerificationConfirmationRequest, EmailVerificationRequest,
};
use crate::email_verifications::service;
use crate::http::error::{AppError, Problem};
use crate::http::rate_limit::{Limited, Policy, Rule, Scope};
use crate::http::validation::ValidJson;

const MINUTE: Duration = Duration::from_secs(60);

pub struct RequestLimit;

impl Policy for RequestLimit {
    const RULES: &'static [Rule] = &[Rule {
        name: "email-verification",
        scope: Scope::Ip,
        window: MINUTE,
        max: 3,
    }];
}

pub struct ConfirmationLimit;

impl Policy for ConfirmationLimit {
    const RULES: &'static [Rule] = &[
        Rule {
            name: "email-confirmation",
            scope: Scope::Ip,
            window: MINUTE,
            max: 10,
        },
        Rule {
            name: "email-confirmation",
            scope: Scope::Network,
            window: MINUTE,
            max: 300,
        },
    ];
}

#[utoipa::path(
    post,
    path = "/email-verifications",
    tag = "email-verifications",
    request_body = EmailVerificationRequest,
    responses(
        (status = 202, description = "Accepted the same way whether or not the email has an unverified account"),
        (status = 400, description = "The body breaks the contract", body = Problem),
        (status = 403, description = "The origin may not change state", body = Problem),
        (status = 429, description = "Too many requests from this client", body = Problem),
        (status = 503, description = "The limit could not be counted", body = Problem),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    _: Limited<RequestLimit>,
    ValidJson(request): ValidJson<EmailVerificationRequest>,
) -> Result<StatusCode, AppError> {
    service::request(&state, &request.email).await?;
    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    post,
    path = "/email-verifications/confirmation",
    tag = "email-verifications",
    request_body = EmailVerificationConfirmationRequest,
    responses(
        (status = 204, description = "The email is verified; no session is opened"),
        (status = 400, description = "The body breaks the contract, or the token is unknown, consumed or expired", body = Problem),
        (status = 403, description = "The origin may not change state", body = Problem),
        (status = 429, description = "Too many requests from this client or its network", body = Problem),
        (status = 503, description = "The limit could not be counted", body = Problem),
    )
)]
pub async fn confirm(
    State(state): State<AppState>,
    _: Limited<ConfirmationLimit>,
    ValidJson(request): ValidJson<EmailVerificationConfirmationRequest>,
) -> Result<StatusCode, AppError> {
    service::confirm(&state, &request.token).await?;
    Ok(StatusCode::NO_CONTENT)
}

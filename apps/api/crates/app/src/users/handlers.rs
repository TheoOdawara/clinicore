use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;

use crate::AppState;
use crate::http::error::{AppError, Problem};
use crate::http::rate_limit::{Limited, Policy, Rule, Scope};
use crate::http::validation::ValidJson;
use crate::users::requests::SignUpRequest;
use crate::users::service;

pub struct SignUpLimit;

impl Policy for SignUpLimit {
    const RULES: &'static [Rule] = &[Rule {
        name: "sign-up",
        scope: Scope::Ip,
        window: Duration::from_secs(60),
        max: 3,
    }];
}

#[utoipa::path(
    post,
    path = "/users",
    tag = "users",
    request_body = SignUpRequest,
    responses(
        (status = 202, description = "Accepted the same way whether or not the email has an account"),
        (status = 400, description = "The body breaks the contract", body = Problem),
        (status = 403, description = "The origin may not change state", body = Problem),
        (status = 429, description = "Too many requests from this client", body = Problem),
        (status = 503, description = "The limit could not be counted", body = Problem),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    _: Limited<SignUpLimit>,
    ValidJson(request): ValidJson<SignUpRequest>,
) -> Result<StatusCode, AppError> {
    service::sign_up(&state, request).await?;
    Ok(StatusCode::ACCEPTED)
}

mod requests;

use api_app::Services;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;

use crate::error::ApiError;
use crate::middleware::rate_limit::{self, Limit};
use crate::validation::ValidJson;
use requests::{EmailRequest, SignUpRequest};

pub fn routes(services: &Services) -> Router {
    let per_minute = |route, count| {
        axum::middleware::from_fn_with_state(
            Limit::per_minute(services, route, count),
            rate_limit::guard,
        )
    };
    Router::new()
        .route(
            "/users",
            post(sign_up).route_layer(per_minute("sign-up", 3)),
        )
        .route(
            "/email-verifications",
            post(request_email_verification).route_layer(per_minute("email-verification", 3)),
        )
        .with_state(services.clone())
}

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
    State(services): State<Services>,
    ValidJson(body): ValidJson<SignUpRequest>,
) -> Result<StatusCode, ApiError> {
    services
        .auth
        .sign_up(&body.name, &body.email, &body.password)
        .await
        .map_err(ApiError)?;
    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    post,
    path = "/email-verifications",
    tag = "Auth",
    summary = "Send a new email verification link",
    description = "Always answers 202 with an empty body. At most one link per address every 60 seconds and five every 24 hours.",
    request_body = EmailRequest,
    responses((status = 202, description = "Accepted, with no body"))
)]
pub async fn request_email_verification(
    State(services): State<Services>,
    ValidJson(body): ValidJson<EmailRequest>,
) -> Result<StatusCode, ApiError> {
    services
        .auth
        .request_email_verification(&body.email)
        .await
        .map_err(ApiError)?;
    Ok(StatusCode::ACCEPTED)
}

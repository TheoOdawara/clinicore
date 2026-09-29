use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::CookieJar;

use super::requests::{
    EmailConfirmationRequest, EmailVerificationRequest, SignInRequest, TokenRefreshRequest,
};
use super::responses::{SessionResponse, SignInResponse, TokenRefreshResponse};
use super::session::{CurrentSession, Device};
use super::{cookies, queries, service};
use crate::AppState;
use crate::http::client::SessionClient;
use crate::http::error::AppError;
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
    summary = "Confirm the email and open a web session",
    description = "Answers 204 with the two session cookies, whatever the Clinicore-Client header says.",
    request_body = EmailConfirmationRequest,
    responses(
        (status = 204, description = "Confirmed, with the session cookies"),
        (status = 400, description = "invalid-token or token-expired")
    )
)]
pub async fn confirm_email(
    State(state): State<AppState>,
    device: Device,
    jar: CookieJar,
    ValidJson(body): ValidJson<EmailConfirmationRequest>,
) -> Result<(StatusCode, CookieJar), AppError> {
    let tokens = service::confirm_email(&state, &body.token, device).await?;
    Ok((
        StatusCode::NO_CONTENT,
        cookies::issue(jar, &tokens, state.secure_cookies),
    ))
}

#[utoipa::path(
    post,
    path = "/sessions",
    tag = "Auth",
    summary = "Sign in with email and password",
    description = "The web gets the session in two cookies. With Clinicore-Client: mobile the tokens come in the body and no cookie is set.",
    request_body = SignInRequest,
    params(("Clinicore-Client" = Option<String>, Header, description = "mobile for the token transport")),
    responses(
        (status = 201, description = "Signed in", body = SignInResponse),
        (status = 401, description = "invalid-credentials"),
        (status = 403, description = "email-not-verified, with a new link sent")
    )
)]
pub async fn sign_in(
    State(state): State<AppState>,
    client: SessionClient,
    device: Device,
    jar: CookieJar,
    ValidJson(body): ValidJson<SignInRequest>,
) -> Result<Response, AppError> {
    let (user, tokens) =
        service::sign_in(&state, &body.email, &body.password, client, device).await?;
    let location = [(LOCATION, "/sessions/current")];
    let response = match client {
        SessionClient::Web => (
            StatusCode::CREATED,
            location,
            cookies::issue(jar, &tokens, state.secure_cookies),
            Json(SessionResponse { user }),
        )
            .into_response(),
        SessionClient::Mobile => (
            StatusCode::CREATED,
            location,
            Json(SignInResponse { user, tokens }),
        )
            .into_response(),
    };
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/sessions/current",
    tag = "Auth",
    summary = "Read the signed-in user",
    params(("Clinicore-Client" = Option<String>, Header, description = "mobile for the token transport")),
    responses(
        (status = 200, description = "The session's user", body = SessionResponse),
        (status = 401, description = "invalid-session")
    )
)]
pub async fn current(
    State(state): State<AppState>,
    session: CurrentSession,
) -> Result<Json<SessionResponse>, AppError> {
    let user = queries::find_session_user(&state.pool, session.user_id)
        .await?
        .ok_or(AppError::InvalidSession)?;
    Ok(Json(SessionResponse { user }))
}

#[utoipa::path(
    delete,
    path = "/sessions/current",
    tag = "Auth",
    summary = "Sign out and revoke the session at once",
    params(("Clinicore-Client" = Option<String>, Header, description = "mobile for the token transport")),
    responses(
        (status = 204, description = "Signed out; the web cookies are cleared"),
        (status = 401, description = "invalid-session")
    )
)]
pub async fn sign_out(
    State(state): State<AppState>,
    session: CurrentSession,
    jar: CookieJar,
) -> Result<Response, AppError> {
    if !queries::close_session(&state.pool, &state.redis, session.session_id).await? {
        return Err(AppError::InvalidSession);
    }
    let response = match session.client {
        SessionClient::Web => (
            StatusCode::NO_CONTENT,
            cookies::clear(jar, state.secure_cookies),
        )
            .into_response(),
        SessionClient::Mobile => StatusCode::NO_CONTENT.into_response(),
    };
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/sessions/current/tokens",
    tag = "Auth",
    summary = "Rotate the refresh token",
    description = "The web sends the refresh cookie and no body, and gets 204 with new cookies. With Clinicore-Client: mobile the refresh token goes in the body and the new tokens come back in it.",
    request_body = Option<TokenRefreshRequest>,
    params(("Clinicore-Client" = Option<String>, Header, description = "mobile for the token transport")),
    responses(
        (status = 200, description = "Rotated, for mobile", body = TokenRefreshResponse),
        (status = 204, description = "Rotated, for the web, with new cookies"),
        (status = 401, description = "invalid-session, or session-reused when an old refresh token comes back")
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    client: SessionClient,
    jar: CookieJar,
    body: Result<ValidJson<TokenRefreshRequest>, AppError>,
) -> Result<Response, AppError> {
    let response = match client {
        SessionClient::Web => {
            let presented = jar.get(cookies::REFRESH).map(|cookie| cookie.value());
            let tokens = service::refresh(&state, presented).await?;
            (
                StatusCode::NO_CONTENT,
                cookies::issue(jar, &tokens, state.secure_cookies),
            )
                .into_response()
        }
        SessionClient::Mobile => {
            let ValidJson(body) = body?;
            let tokens = service::refresh(&state, Some(&body.refresh_token)).await?;
            Json(TokenRefreshResponse { tokens }).into_response()
        }
    };
    Ok(response)
}

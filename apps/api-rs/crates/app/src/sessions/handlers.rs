use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::CookieJar;

use super::extractors::{CurrentSession, Device};
use super::requests::{SignInRequest, TokenRefreshRequest};
use super::responses::{SessionResponse, SignInResponse, TokenRefreshResponse};
use super::service;
use super::tokens::cookies;
use crate::AppState;
use crate::http::client::SessionClient;
use crate::http::error::AppError;
use crate::http::validation::ValidJson;

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
    let user = service::read_current_session(&state, &session).await?;
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
    client: SessionClient,
    session: Result<CurrentSession, AppError>,
    jar: CookieJar,
) -> Response {
    let outcome = match session {
        Ok(session) => service::sign_out(&state, &session)
            .await
            .map_err(AppError::from),
        Err(error) => Err(error),
    };
    match (client, outcome) {
        (SessionClient::Web, Ok(())) => (
            StatusCode::NO_CONTENT,
            cookies::clear(jar, state.secure_cookies),
        )
            .into_response(),
        (SessionClient::Mobile, Ok(())) => StatusCode::NO_CONTENT.into_response(),
        (SessionClient::Web, Err(error)) => refused_on_the_web(jar, &state, error),
        (SessionClient::Mobile, Err(error)) => error.into_response(),
    }
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
            match service::refresh(&state, client, presented).await {
                Ok(tokens) => (
                    StatusCode::NO_CONTENT,
                    cookies::issue(jar, &tokens, state.secure_cookies),
                )
                    .into_response(),
                Err(error) => refused_on_the_web(jar, &state, error.into()),
            }
        }
        SessionClient::Mobile => {
            let ValidJson(body) = body?;
            let tokens = service::refresh(&state, client, Some(&body.refresh_token)).await?;
            Json(TokenRefreshResponse { tokens }).into_response()
        }
    };
    Ok(response)
}

fn refused_on_the_web(jar: CookieJar, state: &AppState, error: AppError) -> Response {
    match error {
        AppError::InvalidSession | AppError::SessionReused => {
            (cookies::clear(jar, state.secure_cookies), error).into_response()
        }
        error => error.into_response(),
    }
}

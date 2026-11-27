use std::time::Duration;

use axum::Json;
use axum::extract::{FromRequest, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use clinicore_core::config::AppEnv;

use crate::AppState;
use crate::http::client::Client;
use crate::http::client_ip::ClientIp;
use crate::http::error::{AppError, Problem};
use crate::http::rate_limit::{Limited, Policy, Rule, Scope};
use crate::http::validation::ValidJson;
use crate::sessions::cookies;
use crate::sessions::extractors::CurrentSession;
use crate::sessions::requests::{RefreshTokensRequest, SignInRequest};
use crate::sessions::responses::{RefreshTokensResponse, SessionResponse, TokensResponse};
use crate::sessions::service::{self, Issued, Origin};
use crate::sessions::tokens::refresh::RefreshToken;

const CURRENT_PATH: &str = "/sessions/current";

pub struct SignInLimit;

impl Policy for SignInLimit {
    const RULES: &'static [Rule] = &[Rule {
        name: "sign-in",
        scope: Scope::Ip,
        window: Duration::from_secs(60),
        max: 5,
    }];
}

pub struct RefreshLimit;

impl Policy for RefreshLimit {
    const RULES: &'static [Rule] = &[Rule {
        name: "session-refresh",
        scope: Scope::Ip,
        window: Duration::from_secs(60),
        max: 30,
    }];
}

pub struct CurrentLimit;

impl Policy for CurrentLimit {
    const RULES: &'static [Rule] = &[Rule {
        name: "session-current",
        scope: Scope::Ip,
        window: Duration::from_secs(10),
        max: 100,
    }];
}

#[utoipa::path(
    post,
    path = "/sessions",
    tag = "sessions",
    request_body = SignInRequest,
    responses(
        (status = 201, description = "The session is open; the web gets two cookies, the app gets the tokens in the body", body = SessionResponse),
        (status = 400, description = "The body breaks the contract, or the client is unknown", body = Problem),
        (status = 401, description = "The email or the password is wrong", body = Problem),
        (status = 403, description = "The email is not verified, or the origin may not change state", body = Problem),
        (status = 429, description = "Too many attempts from this client or for this email", body = Problem),
        (status = 503, description = "The limit or the revocation could not reach Redis", body = Problem),
    )
)]
pub async fn create(
    State(state): State<AppState>,
    client: Client,
    _: Limited<SignInLimit>,
    client_ip: ClientIp,
    headers: HeaderMap,
    ValidJson(request): ValidJson<SignInRequest>,
) -> Result<Response, AppError> {
    let origin = Origin {
        client,
        client_ip: &client_ip,
        user_agent: headers
            .get(header::USER_AGENT)
            .and_then(|agent| agent.to_str().ok()),
    };
    let opened = service::sign_in(&state, origin, request).await?;
    let (cookies, tokens) = deliver(&state, client, opened.issued);
    let body = SessionResponse {
        user: opened.user,
        tokens,
    };
    let response = (
        StatusCode::CREATED,
        [(header::LOCATION, CURRENT_PATH)],
        Json(body),
    );
    Ok(with_cookies(response.into_response(), cookies))
}

#[utoipa::path(
    get,
    path = "/sessions/current",
    tag = "sessions",
    responses(
        (status = 200, description = "The user of the session", body = SessionResponse),
        (status = 401, description = "The session is missing, invalid, expired or revoked", body = Problem),
        (status = 429, description = "Too many requests from this client or this session", body = Problem),
        (status = 503, description = "The limit or the revocation could not reach Redis", body = Problem),
    )
)]
pub async fn show(
    State(state): State<AppState>,
    _: Limited<CurrentLimit>,
    session: CurrentSession,
) -> Result<Json<SessionResponse>, AppError> {
    let user = service::current_user(&state, session.user_id).await?;
    Ok(Json(SessionResponse { user, tokens: None }))
}

#[utoipa::path(
    delete,
    path = "/sessions/current",
    tag = "sessions",
    responses(
        (status = 204, description = "The session is revoked and deleted; the web gets both cookies expired"),
        (status = 401, description = "The session is missing, invalid, expired or revoked", body = Problem),
        (status = 403, description = "The origin may not change state", body = Problem),
        (status = 429, description = "Too many requests from this client or this session", body = Problem),
        (status = 503, description = "The limit or the revocation could not reach Redis", body = Problem),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    _: Limited<CurrentLimit>,
    session: CurrentSession,
) -> Result<Response, AppError> {
    service::close(&state, session.session_id).await?;
    let cookies = match session.client {
        Client::Web => Some(cookies::expired(is_secure(&state))),
        Client::Mobile => None,
    };
    Ok(with_cookies(
        StatusCode::NO_CONTENT.into_response(),
        cookies,
    ))
}

#[utoipa::path(
    post,
    path = "/sessions/current/tokens",
    tag = "sessions",
    request_body(content = RefreshTokensRequest, description = "Only with Clinicore-Client: mobile; the web sends the clinicore_refresh cookie and no body"),
    responses(
        (status = 200, description = "The app gets the new tokens", body = RefreshTokensResponse),
        (status = 204, description = "The web gets two new cookies"),
        (status = 400, description = "The body breaks the contract, or the client is unknown", body = Problem),
        (status = 401, description = "The session is invalid, or the refresh token was already rotated", body = Problem),
        (status = 403, description = "The origin may not change state", body = Problem),
        (status = 429, description = "Too many requests from this client or this session", body = Problem),
        (status = 503, description = "The limit or the revocation could not reach Redis", body = Problem),
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    client: Client,
    _: Limited<RefreshLimit>,
    request: Request,
) -> Result<Response, AppError> {
    let presented = match client {
        Client::Web => cookies::read(request.headers(), cookies::REFRESH)
            .and_then(RefreshToken::parse)
            .ok_or(AppError::InvalidSession)?,
        Client::Mobile => {
            let ValidJson(body) =
                ValidJson::<RefreshTokensRequest>::from_request(request, &state).await?;
            body.refresh_token
        }
    };
    let issued = service::refresh(&state, client, presented).await?;
    let response = match deliver(&state, client, issued) {
        (cookies, None) => with_cookies(StatusCode::NO_CONTENT.into_response(), cookies),
        (_, Some(tokens)) => Json(RefreshTokensResponse { tokens }).into_response(),
    };
    Ok(response)
}

fn deliver(
    state: &AppState,
    client: Client,
    issued: Issued,
) -> (Option<[HeaderValue; 2]>, Option<TokensResponse>) {
    match client {
        Client::Web => (
            Some(cookies::issued(
                &issued.access_token,
                &issued.refresh_token,
                is_secure(state),
            )),
            None,
        ),
        Client::Mobile => (
            None,
            Some(TokensResponse::new(
                issued.access_token,
                &issued.refresh_token,
            )),
        ),
    }
}

fn with_cookies(mut response: Response, cookies: Option<[HeaderValue; 2]>) -> Response {
    for cookie in cookies.into_iter().flatten() {
        response.headers_mut().append(header::SET_COOKIE, cookie);
    }
    response
}

fn is_secure(state: &AppState) -> bool {
    state.config.app_env == AppEnv::Production
}

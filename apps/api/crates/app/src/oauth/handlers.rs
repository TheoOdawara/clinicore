use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::CookieJar;

use super::requests::CallbackQuery;
use super::{cookies, service};
use crate::AppState;
use crate::http::client::ClientAddress;
use crate::http::error::AppError;
use crate::sessions::extractors::Device;
use crate::sessions::tokens::cookies as session_cookies;

#[utoipa::path(
    get,
    path = "/oauth/google",
    tag = "Auth",
    summary = "Start the Google sign-in",
    description = "A top-level navigation, never an XHR. Writes nothing to the database: the state, the PKCE verifier and the nonce travel in three cookies.",
    responses(
        (status = 302, description = "To the Google authorization URL", headers(
            ("Location" = String, description = "The Google authorization URL"),
            ("Set-Cookie" = String, description = "clinicore_oauth_state, clinicore_oauth_verifier and clinicore_oauth_nonce")
        ))
    )
)]
pub async fn start_google_sign_in(
    State(state): State<AppState>,
    client: ClientAddress,
    jar: CookieJar,
) -> Result<Response, AppError> {
    let authorization = service::start(&state, client).await?;
    let jar = cookies::issue(jar, &authorization, state.secure_cookies);
    Ok((StatusCode::FOUND, [(LOCATION, authorization.url)], jar).into_response())
}

#[utoipa::path(
    get,
    path = "/oauth/google/callback",
    tag = "Auth",
    summary = "Finish the Google sign-in",
    description = "Links Google to the account that owns the verified email, or creates the account, and opens a web session. A refusal goes to /login?error= with INVALID_STATE, UNVERIFIED_PROVIDER_EMAIL or PROVIDER_ERROR.",
    params(CallbackQuery),
    responses(
        (status = 302, description = "To /app with the session, or to /login?error= with the refusal", headers(
            ("Location" = String, description = "APP_ORIGIN/app or APP_ORIGIN/login?error=<code>"),
            ("Set-Cookie" = String, description = "clinicore_access and clinicore_refresh when signed in; the three OAuth cookies cleared")
        ))
    )
)]
pub async fn complete_google_sign_in(
    State(state): State<AppState>,
    device: Device,
    jar: CookieJar,
    query: Result<Query<CallbackQuery>, QueryRejection>,
) -> Response {
    let Query(query) = query.unwrap_or_default();
    let issued_state = jar.get(cookies::STATE).map(|cookie| cookie.value());
    let verifier = jar.get(cookies::VERIFIER).map(|cookie| cookie.value());
    let nonce = jar.get(cookies::NONCE).map(|cookie| cookie.value());
    let outcome = service::complete(&state, device, query, issued_state, verifier, nonce).await;

    let jar = cookies::clear(jar, state.secure_cookies);
    let (jar, location) = match outcome {
        Ok(tokens) => (
            session_cookies::issue(jar, &tokens, state.secure_cookies),
            format!("{}/app", state.app_origin),
        ),
        Err(error) => {
            let cause = error.to_string();
            let code = match error.redirect_code() {
                Ok(code) => code,
                Err(error) => return (jar, error).into_response(),
            };
            tracing::warn!(cause, code, "google sign-in refused");
            (jar, format!("{}/login?error={code}", state.app_origin))
        }
    };
    (StatusCode::FOUND, [(LOCATION, location)], jar).into_response()
}

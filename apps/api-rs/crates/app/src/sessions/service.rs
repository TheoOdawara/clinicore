use uuid::Uuid;

use super::error::SessionError;
use super::extractors::{CurrentSession, Device};
use super::queries;
use super::responses::{SessionTokens, SessionUser};
use super::tokens::{access, refresh};
use super::{SESSION_REFRESHES, SIGN_IN_FAILURES};
use crate::AppState;
use crate::credentials::{password, secret};
use crate::email_verifications::service::request_email_verification;
use crate::http::client::SessionClient;
use crate::http::rate_limit;

pub async fn sign_in(
    state: &AppState,
    email: &str,
    password: &str,
    client: SessionClient,
    device: Device,
) -> Result<(SessionUser, SessionTokens), SessionError> {
    let address = email.to_lowercase();
    rate_limit::enforce(&state.redis, SIGN_IN_FAILURES, &address).await?;
    let credential = queries::find_credential(&state.pool, &address)
        .await?
        .and_then(|(user, hash)| hash.map(|hash| (user, hash)));
    let hash = credential
        .as_ref()
        .map_or(state.unmatchable_hash.to_string(), |(_, hash)| hash.clone());

    let matches = password::verify(hash, password.to_string()).await?;
    let Some((user, _)) = credential.filter(|_| matches) else {
        return Err(SessionError::InvalidCredentials);
    };
    rate_limit::refund(&state.redis, SIGN_IN_FAILURES, &address).await?;
    if !user.email_verified {
        request_email_verification(state, &address).await?;
        return Err(SessionError::EmailNotVerified);
    }

    let tokens = open(state, user.id, client, device).await?;
    Ok((user, tokens))
}

pub async fn read_current_session(
    state: &AppState,
    session: &CurrentSession,
) -> Result<SessionUser, SessionError> {
    queries::find_session_user(&state.pool, session.user_id)
        .await?
        .ok_or(SessionError::InvalidSession)
}

pub async fn sign_out(state: &AppState, session: &CurrentSession) -> Result<(), SessionError> {
    queries::close_session(&state.pool, &state.redis, session.session_id).await
}

pub async fn refresh(
    state: &AppState,
    client: SessionClient,
    presented: Option<&str>,
) -> Result<SessionTokens, SessionError> {
    let (session_id, secret) = presented
        .and_then(refresh::parse)
        .ok_or(SessionError::InvalidSession)?;
    rate_limit::enforce(&state.redis, SESSION_REFRESHES, &session_id.to_string()).await?;
    let issued = secret::issue()?;
    let user_id = queries::rotate_session(
        &state.pool,
        &state.redis,
        session_id,
        client,
        &secret::hash(secret),
        &issued.hash,
    )
    .await?;
    tokens(state, user_id, session_id, client, &issued.secret)
}

async fn open(
    state: &AppState,
    user_id: Uuid,
    client: SessionClient,
    device: Device,
) -> Result<SessionTokens, SessionError> {
    let issued = secret::issue()?;
    let session_id = queries::open_session(
        &state.pool,
        &state.redis,
        user_id,
        &issued.hash,
        client,
        device,
    )
    .await?;
    tokens(state, user_id, session_id, client, &issued.secret)
}

fn tokens(
    state: &AppState,
    user_id: Uuid,
    session_id: Uuid,
    client: SessionClient,
    secret: &str,
) -> Result<SessionTokens, SessionError> {
    Ok(SessionTokens {
        access_token: access::sign(&state.access_keys, user_id, session_id, client)?,
        refresh_token: refresh::compose(session_id, secret),
        access_token_expires_in: access::LIFETIME.as_secs(),
    })
}

use uuid::Uuid;

use super::error::AuthError;
use super::queries;
use super::responses::{SessionTokens, SessionUser};
use super::session::{CurrentSession, Device};
use super::{
    EMAIL_CONFIRMATIONS, SESSION_REFRESHES, SIGN_IN_FAILURES, access_token, emails, password, token,
};
use crate::AppState;
use crate::http::client::SessionClient;
use crate::http::rate_limit::{self, Quota};

pub async fn request_email_verification(state: &AppState, email: &str) -> Result<(), AuthError> {
    let address = email.to_lowercase();
    let accepted = queries::reserve_dispatch(&state.pool, &address).await?;
    if !accepted {
        return Ok(());
    }

    let found = queries::find_by_email(&state.pool, &address).await?;
    let Some(user) = found.filter(|user| !user.email_verified) else {
        return Ok(());
    };

    let issued = token::issue()?;
    let message = emails::verification(&user.name, &verification_link(state, &issued.secret))?;
    queries::create_verification(&state.pool, &address, &issued.hash).await?;
    state.mailer.send(&address, message);
    Ok(())
}

pub fn verification_link(state: &AppState, secret: &str) -> String {
    format!("{}/verify-email?token={secret}", state.app_origin)
}

pub async fn confirm_email(state: &AppState, secret: &str) -> Result<(), AuthError> {
    admit(state, EMAIL_CONFIRMATIONS, "all").await?;
    queries::consume_email_verification(&state.pool, &token::hash(secret)).await
}

pub async fn sign_in(
    state: &AppState,
    email: &str,
    password: &str,
    client: SessionClient,
    device: Device,
) -> Result<(SessionUser, SessionTokens), AuthError> {
    let address = email.to_lowercase();
    admit(state, SIGN_IN_FAILURES, &address).await?;
    let credential = queries::find_credential(&state.pool, &address)
        .await?
        .and_then(|(user, hash)| hash.map(|hash| (user, hash)));
    let hash = credential
        .as_ref()
        .map_or(state.unmatchable_hash.to_string(), |(_, hash)| hash.clone());

    let matches = password::verify(hash, password.to_string()).await?;
    let Some((user, _)) = credential.filter(|_| matches) else {
        return Err(AuthError::InvalidCredentials);
    };
    rate_limit::refund(&state.redis, SIGN_IN_FAILURES, &address).await?;
    if !user.email_verified {
        request_email_verification(state, &address).await?;
        return Err(AuthError::EmailNotVerified);
    }

    let tokens = open(state, user.id, client, device).await?;
    Ok((user, tokens))
}

pub async fn read_current_session(
    state: &AppState,
    session: &CurrentSession,
) -> Result<SessionUser, AuthError> {
    queries::find_session_user(&state.pool, session.user_id)
        .await?
        .ok_or(AuthError::InvalidSession)
}

pub async fn sign_out(state: &AppState, session: &CurrentSession) -> Result<(), AuthError> {
    queries::close_session(&state.pool, &state.redis, session.session_id).await
}

pub async fn refresh(
    state: &AppState,
    client: SessionClient,
    presented: Option<&str>,
) -> Result<SessionTokens, AuthError> {
    let (session_id, secret) = presented
        .and_then(token::parse_refresh_token)
        .ok_or(AuthError::InvalidSession)?;
    admit(state, SESSION_REFRESHES, &session_id.to_string()).await?;
    let issued = token::issue()?;
    let user_id = queries::rotate_session(
        &state.pool,
        &state.redis,
        session_id,
        client,
        &token::hash(secret),
        &issued.hash,
    )
    .await?;
    tokens(state, user_id, session_id, client, &issued.secret)
}

async fn admit(state: &AppState, quota: Quota, identity: &str) -> Result<(), AuthError> {
    if !rate_limit::admit(&state.redis, quota, identity).await? {
        return Err(AuthError::RateLimited);
    }
    Ok(())
}

async fn open(
    state: &AppState,
    user_id: Uuid,
    client: SessionClient,
    device: Device,
) -> Result<SessionTokens, AuthError> {
    let issued = token::issue()?;
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
) -> Result<SessionTokens, AuthError> {
    Ok(SessionTokens {
        access_token: access_token::sign(&state.access_keys, user_id, session_id, client)?,
        refresh_token: token::refresh_token(session_id, secret),
        access_token_expires_in: access_token::LIFETIME.as_secs(),
    })
}

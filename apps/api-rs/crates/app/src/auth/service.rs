use uuid::Uuid;

use super::error::AuthError;
use super::queries::{self, Confirmation, Rotation};
use super::responses::{SessionTokens, SessionUser};
use super::session::Device;
use super::{access_token, emails, password, token};
use crate::AppState;
use crate::http::client::SessionClient;

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

pub async fn confirm_email(
    state: &AppState,
    secret: &str,
    device: Device,
) -> Result<SessionTokens, AuthError> {
    match queries::consume_email_verification(&state.pool, &token::hash(secret)).await? {
        Confirmation::Confirmed { user_id } => {
            open(state, user_id, SessionClient::Web, device).await
        }
        Confirmation::Invalid => Err(AuthError::InvalidToken),
        Confirmation::Expired => Err(AuthError::TokenExpired),
    }
}

pub async fn sign_in(
    state: &AppState,
    email: &str,
    password: &str,
    client: SessionClient,
    device: Device,
) -> Result<(SessionUser, SessionTokens), AuthError> {
    let address = email.to_lowercase();
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
    if !user.email_verified {
        request_email_verification(state, &address).await?;
        return Err(AuthError::EmailNotVerified);
    }

    let tokens = open(state, user.id, client, device).await?;
    Ok((user, tokens))
}

pub async fn refresh(
    state: &AppState,
    presented: Option<&str>,
) -> Result<SessionTokens, AuthError> {
    let (session_id, secret) = presented
        .and_then(token::parse_refresh_token)
        .ok_or(AuthError::InvalidSession)?;
    let issued = token::issue()?;
    let rotation = queries::rotate_session(
        &state.pool,
        &state.redis,
        session_id,
        &token::hash(secret),
        &issued.hash,
    )
    .await?;
    match rotation {
        Rotation::Rotated { user_id, client } => {
            tokens(state, user_id, session_id, client, &issued.secret)
        }
        Rotation::Invalid => Err(AuthError::InvalidSession),
        Rotation::Reused => Err(AuthError::SessionReused),
    }
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

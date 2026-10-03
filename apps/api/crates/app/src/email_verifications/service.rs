use super::error::EmailVerificationError;
use super::{NETWORK_CONFIRMATIONS, emails, queries};
use crate::AppState;
use crate::credentials::secret;
use crate::http::client::ClientAddress;
use crate::http::rate_limit;

pub async fn request_email_verification(
    state: &AppState,
    email: &str,
) -> Result<(), EmailVerificationError> {
    let address = email.to_lowercase();
    if !queries::reserve_dispatch(&state.pool, &address).await? {
        return Ok(());
    }

    let found = queries::find_by_email(&state.pool, &address).await?;
    let Some(user) = found.filter(|user| !user.email_verified) else {
        return Ok(());
    };
    send_verification(state, &address, &user.name).await
}

pub async fn resend_verification(
    state: &AppState,
    address: &str,
    name: &str,
) -> Result<(), EmailVerificationError> {
    if !queries::reserve_dispatch(&state.pool, address).await? {
        return Ok(());
    }
    send_verification(state, address, name).await
}

async fn send_verification(
    state: &AppState,
    address: &str,
    name: &str,
) -> Result<(), EmailVerificationError> {
    let issued = secret::issue()?;
    let message = emails::verification(name, &verification_link(state, &issued.secret))?;
    queries::create_verification(&state.pool, address, &issued.hash).await?;
    state.mailer.send(address, message);
    Ok(())
}

pub fn verification_link(state: &AppState, secret: &str) -> String {
    format!("{}/verify-email?token={secret}", state.app_origin)
}

pub async fn confirm_email(
    state: &AppState,
    client: ClientAddress,
    secret: &str,
) -> Result<(), EmailVerificationError> {
    let network = rate_limit::network_key(client);
    rate_limit::enforce(&state.redis, NETWORK_CONFIRMATIONS, &network).await?;
    queries::consume_email_verification(&state.pool, &secret::hash(secret)).await
}

use super::error::PasswordResetError;
use super::{NETWORK_CONFIRMATIONS, emails, queries};
use crate::AppState;
use crate::credentials::{password, secret};
use crate::http::client::ClientAddress;
use crate::http::rate_limit;

pub async fn request_password_reset(
    state: &AppState,
    email: &str,
) -> Result<(), PasswordResetError> {
    let address = email.to_lowercase();
    let issued = secret::issue();
    let link = format!(
        "{}/reset-password?token={}",
        state.app_origin, issued.secret
    );
    queries::reserve_password_reset(&state.pool, &address, &issued.hash, |name| {
        emails::password_reset(name, &link)
    })
    .await
}

pub async fn confirm_password_reset(
    state: &AppState,
    client: ClientAddress,
    secret: &str,
    new_password: &str,
) -> Result<(), PasswordResetError> {
    let network = rate_limit::network_key(client);
    rate_limit::enforce(&state.redis, NETWORK_CONFIRMATIONS, &network).await?;
    let password_hash = password::hash(new_password.to_string()).await;
    queries::reset_password(
        &state.pool,
        &state.redis,
        &secret::hash(secret),
        &password_hash,
    )
    .await
}

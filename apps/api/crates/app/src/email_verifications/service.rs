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
    let issued = secret::issue()?;
    let link = verification_link(state, &issued.secret);
    queries::request_verification(&state.pool, &address, &issued.hash, |name| {
        emails::verification(name, &link)
    })
    .await
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

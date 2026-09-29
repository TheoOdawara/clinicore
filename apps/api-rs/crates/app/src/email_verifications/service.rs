use super::error::EmailVerificationError;
use super::{EMAIL_CONFIRMATIONS, emails, queries};
use crate::AppState;
use crate::credentials::secret;
use crate::http::rate_limit;

pub async fn request_email_verification(
    state: &AppState,
    email: &str,
) -> Result<(), EmailVerificationError> {
    let address = email.to_lowercase();
    let accepted = queries::reserve_dispatch(&state.pool, &address).await?;
    if !accepted {
        return Ok(());
    }

    let found = queries::find_by_email(&state.pool, &address).await?;
    let Some(user) = found.filter(|user| !user.email_verified) else {
        return Ok(());
    };

    let issued = secret::issue()?;
    let message = emails::verification(&user.name, &verification_link(state, &issued.secret))?;
    queries::create_verification(&state.pool, &address, &issued.hash).await?;
    state.mailer.send(&address, message);
    Ok(())
}

pub fn verification_link(state: &AppState, secret: &str) -> String {
    format!("{}/verify-email?token={secret}", state.app_origin)
}

pub async fn confirm_email(state: &AppState, secret: &str) -> Result<(), EmailVerificationError> {
    rate_limit::enforce(&state.redis, EMAIL_CONFIRMATIONS, "all").await?;
    queries::consume_email_verification(&state.pool, &secret::hash(secret)).await
}

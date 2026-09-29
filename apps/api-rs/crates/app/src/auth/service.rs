use super::error::AuthError;
use super::queries;
use super::{emails, token};
use crate::AppState;

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

use super::error::AuthError;
use super::queries::{self, SignUpOutcome};
use super::{messages, password, token};
use crate::AppState;

pub async fn sign_up(
    state: &AppState,
    name: &str,
    email: &str,
    password: &str,
) -> Result<(), AuthError> {
    let address = email.to_lowercase();
    let password_hash = password::hash(password.to_string()).await?;
    let issued = token::issue()?;
    let message = messages::email_verification(name, &verification_link(state, &issued.secret))?;

    let outcome = queries::create_with_credential_account(
        &state.pool,
        name,
        &address,
        &password_hash,
        &issued.hash,
    )
    .await?;

    if outcome == SignUpOutcome::CreatedWithToken {
        state.mailer.send(&address, message);
    }
    Ok(())
}

pub async fn request_email_verification(state: &AppState, email: &str) -> Result<(), AuthError> {
    let address = email.to_lowercase();
    let accepted = queries::register_verification(&state.pool, &address).await?;
    if !accepted {
        return Ok(());
    }

    let found = queries::find_by_email(&state.pool, &address).await?;
    let Some(user) = found.filter(|user| !user.email_verified) else {
        return Ok(());
    };

    let issued = token::issue()?;
    let message =
        messages::email_verification(&user.name, &verification_link(state, &issued.secret))?;
    queries::create_email_verification(&state.pool, &address, &issued.hash).await?;
    state.mailer.send(&address, message);
    Ok(())
}

fn verification_link(state: &AppState, secret: &str) -> String {
    format!("{}/verify-email?token={secret}", state.app_origin)
}

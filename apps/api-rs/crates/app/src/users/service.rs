use super::queries::{self, SignUpOutcome};
use crate::AppState;
use crate::auth::error::AuthError;
use crate::auth::{emails, password, service::verification_link, token};

pub async fn sign_up(
    state: &AppState,
    name: &str,
    email: &str,
    password: &str,
) -> Result<(), AuthError> {
    let address = email.to_lowercase();
    let password_hash = password::hash(password.to_string()).await?;
    let issued = token::issue()?;
    let message = emails::verification(name, &verification_link(state, &issued.secret))?;

    let outcome =
        queries::create_user(&state.pool, name, &address, &password_hash, &issued.hash).await?;

    if outcome == SignUpOutcome::CreatedWithToken {
        state.mailer.send(&address, message);
    }
    Ok(())
}

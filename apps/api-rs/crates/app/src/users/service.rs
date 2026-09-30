use super::error::UserError;
use super::queries::{self, SignUpOutcome};
use crate::AppState;
use crate::credentials::{password, secret};
use crate::email_verifications::emails;
use crate::email_verifications::service::verification_link;

pub async fn sign_up(
    state: &AppState,
    name: &str,
    email: &str,
    password: &str,
) -> Result<(), UserError> {
    let address = email.to_lowercase();
    let password_hash = password::hash(password.to_string()).await?;
    let issued = secret::issue()?;
    let message = emails::verification(name, &verification_link(state, &issued.secret))?;

    let outcome =
        queries::create_user(&state.pool, name, &address, &password_hash, &issued.hash).await?;

    if outcome == SignUpOutcome::CreatedWithToken {
        state.mailer.send(&address, message);
    }
    Ok(())
}

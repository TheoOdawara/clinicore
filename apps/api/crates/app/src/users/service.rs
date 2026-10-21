use super::PASSWORD_FAILURES;
use super::error::UserError;
use super::queries::{self, SignUpOutcome};
use crate::AppState;
use crate::credentials::password::{self, UNMATCHABLE_HASH};
use crate::credentials::secret;
use crate::email_verifications::emails;
use crate::email_verifications::service::verification_link;
use crate::http::rate_limit;
use crate::sessions::extractors::CurrentSession;

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

pub async fn change_password(
    state: &AppState,
    session: &CurrentSession,
    current_password: &str,
    new_password: &str,
) -> Result<(), UserError> {
    let identity = session.user_id.to_string();
    rate_limit::enforce(&state.redis, PASSWORD_FAILURES, &identity).await?;
    let stored = queries::find_password_hash(&state.pool, session.user_id).await?;
    let compared = stored
        .clone()
        .unwrap_or_else(|| UNMATCHABLE_HASH.to_string());

    let matches = password::verify(compared, current_password.to_string()).await?;
    let Some(current_hash) = stored.filter(|_| matches) else {
        return Err(UserError::InvalidPassword);
    };
    rate_limit::refund(&state.redis, PASSWORD_FAILURES, &identity).await?;

    let new_hash = password::hash(new_password.to_string()).await?;
    queries::change_password(&state.pool, &state.redis, session, &current_hash, &new_hash).await
}

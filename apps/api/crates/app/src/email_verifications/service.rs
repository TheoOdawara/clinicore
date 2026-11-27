use std::time::Duration;

use sqlx::PgConnection;

use crate::AppState;
use crate::credentials::email::Email;
use crate::credentials::secret::Secret;
use crate::email_outbox::{self, DispatchKind};
use crate::email_verifications::queries;
use crate::http::error::AppError;

const VERIFICATION_LIFETIME: Duration = Duration::from_secs(60 * 60);
const FIRST_NAME_LENGTH: usize = 30;

pub async fn issue(
    connection: &mut PgConnection,
    app_origin: &str,
    email: &Email,
    name: &str,
) -> Result<(), sqlx::Error> {
    let secret = Secret::generate();
    queries::insert(connection, email, &secret.sha256(), VERIFICATION_LIFETIME).await?;
    let link = format!("{app_origin}/verify-email?token={}", secret.as_str());
    let rendered = clinicore_core::email::email_verification(&first_name(name), &link);
    email_outbox::enqueue(connection, email, &rendered, VERIFICATION_LIFETIME).await
}

pub async fn request(state: &AppState, email: &Email) -> Result<(), AppError> {
    let mut transaction = state.db.begin().await?;
    if email_outbox::reserve(&mut transaction, email, DispatchKind::EmailVerification).await?
        && let Some(name) = queries::unverified_name(&mut transaction, email).await?
    {
        issue(&mut transaction, &state.config.app_origin, email, &name).await?;
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn confirm(state: &AppState, token: &Secret) -> Result<(), AppError> {
    let token_hash = token.sha256();
    let mut transaction = state.db.begin().await?;
    let Some(email) = queries::email_of(&mut transaction, &token_hash).await? else {
        return Err(AppError::InvalidToken);
    };
    queries::lock_address(&mut transaction, &email).await?;
    let token_state = queries::token_state(&mut transaction, &token_hash).await?;
    if token_state.consumed {
        return Err(AppError::InvalidToken);
    }
    if token_state.expired {
        return Err(AppError::TokenExpired);
    }
    queries::verify(&mut transaction, &email).await?;
    transaction.commit().await?;
    Ok(())
}

fn first_name(name: &str) -> String {
    name.split_whitespace()
        .next()
        .unwrap_or_default()
        .chars()
        .take(FIRST_NAME_LENGTH)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_name_is_the_first_word_cut_at_30_characters() {
        assert_eq!(first_name("Ana Souza"), "Ana");
        assert_eq!(first_name(&"Á".repeat(31)), "Á".repeat(30));
    }
}

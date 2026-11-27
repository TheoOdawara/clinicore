use std::time::Duration;

use sqlx::PgConnection;

use crate::credentials::email::Email;
use crate::credentials::secret::Secret;
use crate::email_outbox;
use crate::email_verifications::queries;

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

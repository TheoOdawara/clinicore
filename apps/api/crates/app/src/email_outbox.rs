use clinicore_core::mail::MailMessage;
use sqlx::PgConnection;

#[derive(Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "verification_purpose", rename_all = "snake_case")]
pub enum VerificationPurpose {
    EmailVerification,
    PasswordReset,
}

pub async fn enqueue_with_token(
    connection: &mut PgConnection,
    email: &str,
    purpose: VerificationPurpose,
    token_hash: &str,
    message: &MailMessage,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "WITH token AS (
            INSERT INTO verifications (email, purpose, token_hash, expires_at)
            VALUES ($1, $2, $3, now() + interval '1 hour')
            RETURNING expires_at
        )
        INSERT INTO email_outbox (email, subject, html_body, text_body, expires_at)
        SELECT $1, $4, $5, $6, expires_at FROM token",
        email,
        purpose as VerificationPurpose,
        token_hash,
        message.subject,
        message.html,
        message.text
    )
    .execute(connection)
    .await?;
    Ok(())
}

use sqlx::PgPool;
use uuid::Uuid;

use super::error::OAuthError;
use super::google::GoogleProfile;

pub async fn link_google_account(
    pool: &PgPool,
    profile: &GoogleProfile,
) -> Result<Uuid, OAuthError> {
    let linked = sqlx::query_scalar!(
        "SELECT user_id FROM accounts WHERE provider = 'google' AND provider_account_id = $1",
        profile.subject
    )
    .fetch_optional(pool)
    .await?;
    if let Some(user_id) = linked {
        return Ok(user_id);
    }

    let email = profile.email.to_lowercase();
    let name = profile.name.as_ref().unwrap_or(&email);
    let mut transaction = pool.begin().await?;
    let user = sqlx::query!(
        "INSERT INTO users (name, email, email_verified, image) VALUES ($1, $2, true, $3)
        ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email
        RETURNING id, email_verified",
        name,
        email,
        profile.picture
    )
    .fetch_one(&mut *transaction)
    .await?;

    if !user.email_verified {
        sqlx::query!(
            "DELETE FROM accounts WHERE user_id = $1 AND provider = 'credential'",
            user.id
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query!(
            "UPDATE verifications SET consumed_at = now()
            WHERE email = $1 AND purpose = 'email_verification' AND consumed_at IS NULL",
            email
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query!(
            "UPDATE users SET email_verified = true, name = $2, image = $3, updated_at = now()
            WHERE id = $1",
            user.id,
            name,
            profile.picture
        )
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query!(
        "INSERT INTO accounts (user_id, provider, provider_account_id) VALUES ($1, 'google', $2)
        ON CONFLICT DO NOTHING",
        user.id,
        profile.subject
    )
    .execute(&mut *transaction)
    .await?;
    let linked = sqlx::query_scalar!(
        r#"SELECT exists(
            SELECT 1 FROM accounts
            WHERE user_id = $1 AND provider = 'google' AND provider_account_id = $2
        ) AS "linked!""#,
        user.id,
        profile.subject
    )
    .fetch_one(&mut *transaction)
    .await?;
    if !linked {
        return Err(OAuthError::LinkedToAnotherAccount);
    }

    transaction.commit().await?;
    Ok(user.id)
}

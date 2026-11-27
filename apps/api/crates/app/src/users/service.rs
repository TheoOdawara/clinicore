use crate::AppState;
use crate::credentials::password;
use crate::email_outbox::{self, DispatchKind};
use crate::email_verifications;
use crate::http::error::AppError;
use crate::users::queries;
use crate::users::requests::SignUpRequest;

pub async fn sign_up(state: &AppState, request: SignUpRequest) -> Result<(), AppError> {
    let password_hash = password::hash(request.password).await;
    let mut transaction = state.db.begin().await?;
    let Some(user_id) =
        queries::insert_user(&mut transaction, &request.name, &request.email).await?
    else {
        return Ok(());
    };
    queries::insert_credential_account(&mut transaction, user_id, &password_hash).await?;
    if email_outbox::reserve(
        &mut transaction,
        &request.email,
        DispatchKind::EmailVerification,
    )
    .await?
    {
        email_verifications::service::issue(
            &mut transaction,
            &state.config.app_origin,
            &request.email,
            &request.name,
        )
        .await?;
    }
    transaction.commit().await?;
    Ok(())
}

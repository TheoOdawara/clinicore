mod messages;
mod password;
mod token;

use api_infra::auth::user::{self, SignUpOutcome};
use api_infra::auth::{email_dispatch, verification};
use api_infra::config::Config;
use api_infra::db::Database;
use api_infra::mail::Mailer;

use crate::error::AppError;

#[derive(Clone)]
pub struct Auth {
    database: Database,
    mailer: Mailer,
    app_origin: String,
}

impl Auth {
    pub fn new(config: &Config, database: Database, mailer: Mailer) -> Self {
        Self {
            database,
            mailer,
            app_origin: config.app_origin.clone(),
        }
    }

    pub async fn sign_up(&self, name: &str, email: &str, password: &str) -> Result<(), AppError> {
        let address = email.to_lowercase();
        let password_hash = password::hash(password.to_string()).await?;
        let issued = token::issue()?;
        let message = messages::email_verification(name, &self.verification_link(&issued.secret))?;

        let outcome = user::create_with_credential_account(
            &self.database,
            name,
            &address,
            &password_hash,
            &issued.hash,
        )
        .await
        .map_err(AppError::internal)?;

        if outcome == SignUpOutcome::CreatedWithToken {
            self.mailer.send(&address, message);
        }
        Ok(())
    }

    pub async fn request_email_verification(&self, email: &str) -> Result<(), AppError> {
        let address = email.to_lowercase();
        let accepted = email_dispatch::register_verification(&self.database, &address)
            .await
            .map_err(AppError::internal)?;
        if !accepted {
            return Ok(());
        }

        let found = user::find_by_email(&self.database, &address)
            .await
            .map_err(AppError::internal)?;
        let Some(user) = found.filter(|user| !user.email_verified) else {
            return Ok(());
        };

        let issued = token::issue()?;
        let message =
            messages::email_verification(&user.name, &self.verification_link(&issued.secret))?;
        verification::create_email_verification(&self.database, &address, &issued.hash)
            .await
            .map_err(AppError::internal)?;
        self.mailer.send(&address, message);
        Ok(())
    }

    fn verification_link(&self, secret: &str) -> String {
        format!("{}/verify-email?token={secret}", self.app_origin)
    }
}

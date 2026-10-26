#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error(transparent)]
    PasswordHash(#[from] argon2::password_hash::Error),
}

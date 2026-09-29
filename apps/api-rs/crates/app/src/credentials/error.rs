#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error(transparent)]
    PasswordHash(#[from] argon2::password_hash::Error),
    #[error(transparent)]
    Random(#[from] getrandom::Error),
    #[error(transparent)]
    Task(#[from] tokio::task::JoinError),
    #[error(transparent)]
    HashingSlot(#[from] tokio::sync::AcquireError),
}

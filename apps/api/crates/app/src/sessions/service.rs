use std::time::Duration;

use subtle::ConstantTimeEq;

use crate::AppState;
use crate::credentials::email::Email;
use crate::credentials::password;
use crate::credentials::secret::Secret;
use crate::email_outbox::{self, DispatchKind};
use crate::email_verifications;
use crate::http::client::Client;
use crate::http::client_ip::ClientIp;
use crate::http::error::AppError;
use crate::http::rate_limit;
use crate::sessions::SessionId;
use crate::sessions::queries::{self, NewSession};
use crate::sessions::requests::SignInRequest;
use crate::sessions::responses::UserResponse;
use crate::sessions::tokens::refresh::{self, RefreshToken};
use crate::sessions::tokens::{access, denylist};
use crate::users::UserId;

const FAILURE_WINDOW: Duration = Duration::from_secs(15 * 60);
const FAILURES_PER_WINDOW: u64 = 10;
const REFRESH_WINDOW: Duration = Duration::from_secs(60);
const REFRESHES_PER_WINDOW: u64 = 30;
const SESSIONS_KEPT_BESIDE_THE_NEW: i64 = 4;
const USER_AGENT_MAX_LENGTH: usize = 512;

pub struct Issued {
    pub access_token: String,
    pub refresh_token: RefreshToken,
}

pub struct Opened {
    pub user: UserResponse,
    pub issued: Issued,
}

pub struct Origin<'a> {
    pub client: Client,
    pub client_ip: &'a ClientIp,
    pub user_agent: Option<&'a str>,
}

pub async fn sign_in(
    state: &AppState,
    origin: Origin<'_>,
    request: SignInRequest,
) -> Result<Opened, AppError> {
    let failures = format!("rate:sign-in-failures:email:{}", request.email.as_str());
    rate_limit::hit(&state.redis, &failures, FAILURE_WINDOW, FAILURES_PER_WINDOW).await?;
    let credential = queries::credential_of(&state.db, &request.email).await?;
    let password_hash = credential
        .as_ref()
        .and_then(|credential| credential.password_hash.clone());
    if !password::verify(request.password, password_hash).await {
        return Err(AppError::InvalidCredentials);
    }
    let credential = credential.expect("a password only verifies against a stored credential");
    rate_limit::refund(&state.redis, &failures, FAILURE_WINDOW).await?;
    if !credential.email_verified {
        resend(state, &request.email, &credential.name).await?;
        return Err(AppError::EmailNotVerified);
    }
    let user = UserResponse {
        id: credential.user_id,
        name: credential.name,
        email: request.email.as_str().to_owned(),
    };
    let issued = open(state, origin, UserId(credential.user_id)).await?;
    Ok(Opened { user, issued })
}

async fn resend(state: &AppState, email: &Email, name: &str) -> Result<(), AppError> {
    let mut transaction = state.db.begin().await?;
    if email_outbox::reserve(&mut transaction, email, DispatchKind::EmailVerification).await? {
        email_verifications::service::issue(
            &mut transaction,
            &state.config.app_origin,
            email,
            name,
        )
        .await?;
    }
    transaction.commit().await?;
    Ok(())
}

async fn open(state: &AppState, origin: Origin<'_>, user_id: UserId) -> Result<Issued, AppError> {
    let secret = Secret::generate();
    let mut transaction = state.db.begin().await?;
    queries::lock_user(&mut transaction, user_id).await?;
    let session_id = queries::insert(
        &mut transaction,
        NewSession {
            user_id,
            client: origin.client,
            refresh_token_hash: &secret.sha256(),
            lifetime: refresh::lifetime(origin.client),
            ip_address: origin.client_ip.address.map(|address| address.to_string()),
            user_agent: origin
                .user_agent
                .map(|agent| agent.chars().take(USER_AGENT_MAX_LENGTH).collect()),
        },
    )
    .await?;
    let evicted = queries::evict_beyond(
        &mut transaction,
        user_id,
        session_id,
        SESSIONS_KEPT_BESIDE_THE_NEW,
    )
    .await?;
    for evicted_id in evicted {
        denylist::revoke(&state.redis, evicted_id).await?;
    }
    transaction.commit().await?;
    Ok(issue(state, user_id, session_id, origin.client, secret))
}

pub async fn refresh(
    state: &AppState,
    client: Client,
    presented: RefreshToken,
) -> Result<Issued, AppError> {
    let session_id = presented.session_id;
    let key = format!("rate:session-refresh:session:{}", session_id.0);
    rate_limit::hit(&state.redis, &key, REFRESH_WINDOW, REFRESHES_PER_WINDOW).await?;
    if denylist::is_revoked(&state.redis, session_id).await? {
        return Err(AppError::InvalidSession);
    }
    let mut transaction = state.db.begin().await?;
    let session = queries::locked(&mut transaction, session_id)
        .await?
        .filter(|session| session.usable && session.client == client)
        .ok_or(AppError::InvalidSession)?;
    let matches: bool = session
        .refresh_token_hash
        .ct_eq(&presented.secret.sha256())
        .into();
    if !matches {
        transaction.rollback().await?;
        close(state, session_id).await?;
        return Err(AppError::SessionReused);
    }
    let secret = Secret::generate();
    queries::rotate(
        &mut transaction,
        session_id,
        &secret.sha256(),
        refresh::lifetime(client),
    )
    .await?;
    transaction.commit().await?;
    Ok(issue(
        state,
        UserId(session.user_id),
        session_id,
        client,
        secret,
    ))
}

pub async fn close(state: &AppState, session_id: SessionId) -> Result<(), AppError> {
    denylist::revoke(&state.redis, session_id).await?;
    queries::delete(&state.db, session_id).await?;
    Ok(())
}

pub async fn current_user(state: &AppState, user_id: UserId) -> Result<UserResponse, AppError> {
    let user = queries::user_of(&state.db, user_id)
        .await?
        .ok_or(AppError::InvalidSession)?;
    Ok(UserResponse {
        id: user.id,
        name: user.name,
        email: user.email,
    })
}

fn issue(
    state: &AppState,
    user_id: UserId,
    session_id: SessionId,
    client: Client,
    secret: Secret,
) -> Issued {
    Issued {
        access_token: access::issue(
            &state.config.jwt_secret,
            user_id,
            session_id,
            client,
            access::now(),
        ),
        refresh_token: RefreshToken { session_id, secret },
    }
}

use super::NETWORK_REQUESTS;
use super::error::OAuthError;
use super::google::Authorization;
use super::queries;
use super::requests::CallbackQuery;
use crate::AppState;
use crate::credentials::secret;
use crate::http::client::{ClientAddress, SessionClient};
use crate::http::rate_limit::{self, RateLimitError};
use crate::sessions::extractors::Device;
use crate::sessions::responses::SessionTokens;
use crate::sessions::service::open_session;

pub async fn start(
    state: &AppState,
    client: ClientAddress,
) -> Result<Authorization, RateLimitError> {
    let network = rate_limit::network_key(client);
    rate_limit::enforce(&state.redis, NETWORK_REQUESTS, &network).await?;
    Ok(state.google.authorization())
}

pub async fn complete(
    state: &AppState,
    device: Device,
    query: CallbackQuery,
    issued_state: Option<&str>,
    verifier: Option<&str>,
    nonce: Option<&str>,
) -> Result<SessionTokens, OAuthError> {
    let network = rate_limit::network_key(ClientAddress(device.ip_address));
    rate_limit::enforce(&state.redis, NETWORK_REQUESTS, &network).await?;

    let (Some(returned_state), Some(issued_state), Some(verifier), Some(nonce)) =
        (query.state, issued_state, verifier, nonce)
    else {
        return Err(OAuthError::InvalidState);
    };
    if secret::hash(&returned_state) != secret::hash(issued_state) {
        return Err(OAuthError::InvalidState);
    }
    let Some(code) = query.code.filter(|_| query.error.is_none()) else {
        return Err(OAuthError::AuthorizationDenied);
    };

    let profile = state
        .google
        .read_profile(code, verifier.to_string(), nonce.to_string())
        .await?;
    if !profile.email_verified {
        return Err(OAuthError::UnverifiedProviderEmail);
    }
    let user_id = queries::link_google_account(&state.pool, &profile).await?;
    Ok(open_session(state, user_id, SessionClient::Web, device).await?)
}

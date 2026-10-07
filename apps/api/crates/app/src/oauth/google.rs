use std::time::Duration;

use clinicore_core::config::Config;
use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreIdTokenVerifier, CoreJsonWebKeySet,
    CoreRequestTokenError,
};
use openidconnect::url::ParseError;
use openidconnect::{
    AuthUrl, AuthorizationCode, ClaimsVerificationError, ClientId, ClientSecret, CsrfToken,
    DiscoveryError, EndpointNotSet, EndpointSet, HttpClientError, IssuerUrl, JsonWebKeySetUrl,
    Nonce, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
    reqwest,
};

#[derive(Clone)]
pub struct GoogleEndpoints {
    pub issuer: String,
    pub authorization: String,
    pub token: String,
    pub keys: String,
}

impl GoogleEndpoints {
    pub fn production() -> Self {
        Self {
            issuer: "https://accounts.google.com".to_string(),
            authorization: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token: "https://oauth2.googleapis.com/token".to_string(),
            keys: "https://www.googleapis.com/oauth2/v3/certs".to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GoogleClientError {
    #[error(transparent)]
    Url(#[from] ParseError),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum GoogleError {
    #[error(transparent)]
    CodeExchange(#[from] CoreRequestTokenError<HttpClientError<reqwest::Error>>),
    #[error("the token response carries no ID token")]
    MissingIdToken,
    #[error(transparent)]
    Keys(#[from] DiscoveryError<HttpClientError<reqwest::Error>>),
    #[error(transparent)]
    Claims(#[from] ClaimsVerificationError),
    #[error("the ID token carries no email")]
    MissingEmail,
}

pub struct Authorization {
    pub url: String,
    pub state: String,
    pub verifier: String,
    pub nonce: String,
}

pub struct GoogleProfile {
    pub subject: String,
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Clone)]
pub struct GoogleClient {
    flow: CoreClient<
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointSet,
        EndpointNotSet,
    >,
    http: reqwest::Client,
    issuer: IssuerUrl,
    keys: JsonWebKeySetUrl,
}

impl GoogleClient {
    pub fn new(config: &Config, endpoints: GoogleEndpoints) -> Result<Self, GoogleClientError> {
        let issuer = IssuerUrl::new(endpoints.issuer)?;
        let callback = format!("{}/oauth/google/callback", config.api_url);
        let flow = CoreClient::new(
            ClientId::new(config.google_client_id.clone()),
            issuer.clone(),
            CoreJsonWebKeySet::new(Vec::new()),
        )
        .set_client_secret(ClientSecret::new(config.google_client_secret.clone()))
        .set_auth_uri(AuthUrl::new(endpoints.authorization)?)
        .set_token_uri(TokenUrl::new(endpoints.token)?)
        .set_redirect_uri(RedirectUrl::new(callback)?);
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            flow,
            http,
            issuer,
            keys: JsonWebKeySetUrl::new(endpoints.keys)?,
        })
    }

    pub fn authorization(&self) -> Authorization {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = self
            .flow
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                || CsrfToken::new_random_len(32),
                || Nonce::new_random_len(32),
            )
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_extra_param("prompt", "select_account")
            .set_pkce_challenge(challenge)
            .url();
        Authorization {
            url: url.into(),
            state: state.into_secret(),
            verifier: verifier.into_secret(),
            nonce: nonce.secret().clone(),
        }
    }

    pub async fn read_profile(
        &self,
        code: String,
        verifier: String,
        nonce: String,
    ) -> Result<GoogleProfile, GoogleError> {
        // ponytail: fetches Google's signing keys on every sign-in, so a sign-in costs one more request and fails while that endpoint is down; cache them by the response's Cache-Control max-age, refetching on an unknown key id, when either matters
        let keys = CoreJsonWebKeySet::fetch_async(&self.keys, &self.http).await?;
        let token = self
            .flow
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier))
            .request_async(&self.http)
            .await?;
        let id_token = token.id_token().ok_or(GoogleError::MissingIdToken)?;
        let id_token_verifier = CoreIdTokenVerifier::new_public_client(
            self.flow.client_id().clone(),
            self.issuer.clone(),
            keys,
        );
        let claims = id_token.claims(&id_token_verifier, &Nonce::new(nonce))?;
        let email = claims.email().ok_or(GoogleError::MissingEmail)?;
        Ok(GoogleProfile {
            subject: claims.subject().to_string(),
            email: email.to_string(),
            email_verified: claims.email_verified().unwrap_or(false),
            name: claims
                .name()
                .and_then(|name| name.get(None))
                .map(|name| name.to_string()),
            picture: claims
                .picture()
                .and_then(|picture| picture.get(None))
                .map(|picture| picture.to_string()),
        })
    }
}

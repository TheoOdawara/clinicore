use std::time::Duration;

use clinicore_core::config::Config;
use oauth2::basic::{BasicClient, BasicRequestTokenError};
use oauth2::url::ParseError;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    HttpClientError, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
    TokenUrl, reqwest,
};
use serde::Deserialize;

#[derive(Clone)]
pub struct GoogleEndpoints {
    pub authorization: String,
    pub token: String,
    pub userinfo: String,
}

impl GoogleEndpoints {
    pub fn production() -> Self {
        Self {
            authorization: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token: "https://oauth2.googleapis.com/token".to_string(),
            userinfo: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
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
    CodeExchange(#[from] BasicRequestTokenError<HttpClientError<reqwest::Error>>),
    #[error(transparent)]
    Userinfo(#[from] reqwest::Error),
    #[error(transparent)]
    Profile(#[from] serde_json::Error),
}

pub struct Authorization {
    pub url: String,
    pub state: String,
    pub verifier: String,
}

#[derive(Deserialize)]
pub struct GoogleProfile {
    #[serde(rename = "sub")]
    pub subject: String,
    pub email: String,
    #[serde(default)]
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Clone)]
pub struct GoogleClient {
    flow: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    http: reqwest::Client,
    userinfo: String,
}

impl GoogleClient {
    pub fn new(config: &Config, endpoints: GoogleEndpoints) -> Result<Self, GoogleClientError> {
        let callback = format!("{}/oauth/google/callback", config.api_url);
        let flow = BasicClient::new(ClientId::new(config.google_client_id.clone()))
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
            userinfo: endpoints.userinfo,
        })
    }

    pub fn authorization(&self) -> Authorization {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state) = self
            .flow
            .authorize_url(|| CsrfToken::new_random_len(32))
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_extra_param("prompt", "select_account")
            .set_pkce_challenge(challenge)
            .url();
        Authorization {
            url: url.into(),
            state: state.into_secret(),
            verifier: verifier.into_secret(),
        }
    }

    pub async fn read_profile(
        &self,
        code: String,
        verifier: String,
    ) -> Result<GoogleProfile, GoogleError> {
        let token = self
            .flow
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(PkceCodeVerifier::new(verifier))
            .request_async(&self.http)
            .await?;
        let profile = self
            .http
            .get(&self.userinfo)
            .bearer_auth(token.access_token().secret())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(serde_json::from_slice(&profile)?)
    }
}

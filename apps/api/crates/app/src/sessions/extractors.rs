use std::time::Duration;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderMap, header};

use crate::AppState;
use crate::http::client::Client;
use crate::http::error::AppError;
use crate::http::rate_limit;
use crate::sessions::tokens::{access, denylist};
use crate::sessions::{SessionId, cookies};
use crate::users::UserId;

const CURRENT_WINDOW: Duration = Duration::from_secs(10);
const CURRENT_PER_WINDOW: u64 = 100;
const BEARER_PREFIX: &str = "Bearer ";

pub struct CurrentSession {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub client: Client,
}

impl FromRequestParts<AppState> for CurrentSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let client = Client::declared(&parts.headers)?;
        let token = match client {
            Client::Web => cookies::read(&parts.headers, cookies::ACCESS),
            Client::Mobile => bearer(&parts.headers),
        }
        .ok_or(AppError::InvalidSession)?;
        let claims = access::verify(&state.config.jwt_secret, token)
            .filter(|claims| claims.cli == client)
            .ok_or(AppError::InvalidSession)?;
        let session_id = SessionId(claims.sid);
        if denylist::is_revoked(&state.redis, session_id).await? {
            return Err(AppError::InvalidSession);
        }
        let key = format!("rate:session-current:session:{}", claims.sid);
        rate_limit::hit(&state.redis, &key, CURRENT_WINDOW, CURRENT_PER_WINDOW).await?;
        Ok(Self {
            user_id: UserId(claims.sub),
            session_id,
            client,
        })
    }
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix(BEARER_PREFIX)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    #[test]
    fn the_bearer_is_read_only_from_a_readable_bearer_header() {
        let mut headers = HeaderMap::new();
        assert_eq!(bearer(&headers), None);

        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer token"),
        );
        assert_eq!(bearer(&headers), Some("token"));

        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_bytes(b"Bearer \xfa").expect("an opaque value"),
        );
        assert_eq!(bearer(&headers), None);
    }
}

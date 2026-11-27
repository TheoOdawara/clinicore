use std::fmt;
use std::time::Duration;

use uuid::Uuid;

use crate::credentials::secret::Secret;
use crate::http::client::Client;
use crate::sessions::SessionId;

const WEB_LIFETIME: Duration = Duration::from_secs(24 * 60 * 60);
const MOBILE_LIFETIME: Duration = Duration::from_secs(7 * 24 * 60 * 60);
pub const LONGEST_LIFETIME: Duration = MOBILE_LIFETIME;
const HYPHENATED_UUID_LENGTH: usize = 36;

pub struct RefreshToken {
    pub session_id: SessionId,
    pub secret: Secret,
}

impl RefreshToken {
    pub fn parse(raw: &str) -> Option<Self> {
        let (session_id, secret) = raw.split_once('.')?;
        if session_id.len() != HYPHENATED_UUID_LENGTH {
            return None;
        }
        Some(Self {
            session_id: SessionId(Uuid::try_parse(session_id).ok()?),
            secret: Secret::parse(secret)?,
        })
    }
}

impl fmt::Display for RefreshToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{}",
            self.session_id.0.hyphenated(),
            self.secret.as_str()
        )
    }
}

pub fn lifetime(client: Client) -> Duration {
    match client {
        Client::Web => WEB_LIFETIME,
        Client::Mobile => MOBILE_LIFETIME,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_the_session_id_and_the_secret_and_parses_back() {
        let token = RefreshToken {
            session_id: SessionId(Uuid::from_u128(7)),
            secret: Secret::generate(),
        };
        let text = token.to_string();

        let parsed = RefreshToken::parse(&text).expect("a written token parses");
        assert_eq!(parsed.session_id, token.session_id);
        assert_eq!(parsed.secret.as_str(), token.secret.as_str());
        assert!(text.starts_with("00000000-0000-0000-0000-000000000007."));
    }

    #[test]
    fn a_malformed_token_is_refused() {
        let secret = "a".repeat(43);
        for raw in [
            secret.clone(),
            format!("00000000000000000000000000000007.{secret}"),
            format!("zzzzzzzz-0000-0000-0000-000000000007.{secret}"),
            "00000000-0000-0000-0000-000000000007.short".to_owned(),
        ] {
            assert!(RefreshToken::parse(&raw).is_none(), "{raw}");
        }
    }

    #[test]
    fn the_web_lives_a_day_and_the_app_a_week() {
        assert_eq!(lifetime(Client::Web), Duration::from_secs(86_400));
        assert_eq!(lifetime(Client::Mobile), Duration::from_secs(604_800));
        assert_eq!(LONGEST_LIFETIME, lifetime(Client::Mobile));
    }
}

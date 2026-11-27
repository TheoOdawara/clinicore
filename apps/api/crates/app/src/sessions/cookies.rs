use axum::http::{HeaderMap, HeaderValue, header};

use crate::http::client::Client;
use crate::sessions::tokens::access::ACCESS_LIFETIME;
use crate::sessions::tokens::refresh::{RefreshToken, lifetime};

pub const ACCESS: &str = "clinicore_access";
pub const REFRESH: &str = "clinicore_refresh";
const ACCESS_PATH: &str = "/";
const REFRESH_PATH: &str = "/sessions/current/tokens";

pub fn issued(access_token: &str, refresh_token: &RefreshToken, secure: bool) -> [HeaderValue; 2] {
    [
        cookie(
            ACCESS,
            access_token,
            ACCESS_PATH,
            ACCESS_LIFETIME.as_secs(),
            secure,
        ),
        cookie(
            REFRESH,
            &refresh_token.to_string(),
            REFRESH_PATH,
            lifetime(Client::Web).as_secs(),
            secure,
        ),
    ]
}

pub fn expired(secure: bool) -> [HeaderValue; 2] {
    [
        cookie(ACCESS, "", ACCESS_PATH, 0, secure),
        cookie(REFRESH, "", REFRESH_PATH, 0, secure),
    ]
}

pub fn read<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|pair| pair.trim().strip_prefix(name)?.strip_prefix('='))
}

fn cookie(name: &str, value: &str, path: &str, max_age: u64, secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{name}={value}; Path={path}; Max-Age={max_age}; HttpOnly; SameSite=Lax{secure}"
    ))
    .expect("a session cookie holds only base64url, dots and hyphens")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cookie_is_read_by_its_exact_name_among_several_headers() {
        let mut headers = HeaderMap::new();
        headers.append(
            header::COOKIE,
            HeaderValue::from_static("a=1; clinicore_access_x=2"),
        );
        headers.append(
            header::COOKIE,
            HeaderValue::from_static("b=3;clinicore_access=token"),
        );

        assert_eq!(read(&headers, ACCESS), Some("token"));
        assert_eq!(read(&headers, REFRESH), None);
    }

    #[test]
    fn secure_is_added_only_when_asked() {
        let [access, _] = expired(true);
        let [plain, _] = expired(false);

        assert!(access.to_str().expect("ascii").ends_with("; Secure"));
        assert!(!plain.to_str().expect("ascii").contains("Secure"));
    }
}

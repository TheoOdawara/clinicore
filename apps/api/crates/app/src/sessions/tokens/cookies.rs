use std::time::Duration;

use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

use super::{access, refresh};
use crate::http::client::SessionClient;
use crate::sessions::responses::SessionTokens;

pub const ACCESS: &str = "clinicore_access";
pub const REFRESH: &str = "clinicore_refresh";

const REFRESH_PATH: &str = "/sessions/current/tokens";

pub fn issue(jar: CookieJar, tokens: &SessionTokens, secure: bool) -> CookieJar {
    jar.add(cookie(
        ACCESS,
        &tokens.access_token,
        "/",
        access::LIFETIME,
        secure,
    ))
    .add(cookie(
        REFRESH,
        &tokens.refresh_token,
        REFRESH_PATH,
        refresh::lifetime(SessionClient::Web),
        secure,
    ))
}

pub fn clear(jar: CookieJar, secure: bool) -> CookieJar {
    jar.add(cookie(ACCESS, "", "/", Duration::ZERO, secure))
        .add(cookie(REFRESH, "", REFRESH_PATH, Duration::ZERO, secure))
}

fn cookie(
    name: &'static str,
    value: &str,
    path: &'static str,
    max_age: Duration,
    secure: bool,
) -> Cookie<'static> {
    Cookie::build((name, value.to_string()))
        .path(path)
        .max_age(max_age.try_into().expect("a lifetime a cookie can carry"))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .build()
}

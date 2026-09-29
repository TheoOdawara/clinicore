use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use cookie::time::Duration;

use super::responses::SessionTokens;

pub const ACCESS: &str = "clinicore_access";
pub const REFRESH: &str = "clinicore_refresh";

pub fn issue(jar: CookieJar, tokens: &SessionTokens, secure: bool) -> CookieJar {
    jar.add(cookie(ACCESS, &tokens.access_token, "/", 900, secure))
        .add(cookie(
            REFRESH,
            &tokens.refresh_token,
            REFRESH_PATH,
            86400,
            secure,
        ))
}

pub fn clear(jar: CookieJar, secure: bool) -> CookieJar {
    jar.add(cookie(ACCESS, "", "/", 0, secure))
        .add(cookie(REFRESH, "", REFRESH_PATH, 0, secure))
}

const REFRESH_PATH: &str = "/sessions/current/tokens";

fn cookie(
    name: &'static str,
    value: &str,
    path: &'static str,
    max_age: i64,
    secure: bool,
) -> Cookie<'static> {
    Cookie::build((name, value.to_string()))
        .path(path)
        .max_age(Duration::seconds(max_age))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .build()
}

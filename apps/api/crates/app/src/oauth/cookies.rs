use std::time::Duration;

use axum_extra::extract::cookie::CookieJar;

use super::google::Authorization;
use crate::sessions::tokens::cookies::cookie;

pub const STATE: &str = "clinicore_oauth_state";
pub const VERIFIER: &str = "clinicore_oauth_verifier";
pub const NONCE: &str = "clinicore_oauth_nonce";

const PATH: &str = "/oauth/google";
const LIFETIME: Duration = Duration::from_secs(600);

pub fn issue(jar: CookieJar, authorization: &Authorization, secure: bool) -> CookieJar {
    jar.add(cookie(STATE, &authorization.state, PATH, LIFETIME, secure))
        .add(cookie(
            VERIFIER,
            &authorization.verifier,
            PATH,
            LIFETIME,
            secure,
        ))
        .add(cookie(NONCE, &authorization.nonce, PATH, LIFETIME, secure))
}

pub fn clear(jar: CookieJar, secure: bool) -> CookieJar {
    jar.add(cookie(STATE, "", PATH, Duration::ZERO, secure))
        .add(cookie(VERIFIER, "", PATH, Duration::ZERO, secure))
        .add(cookie(NONCE, "", PATH, Duration::ZERO, secure))
}

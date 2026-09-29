mod current_session;
mod refresh;
mod sign_in;

use axum::Router;
use axum::http::StatusCode;
use axum::response::Response;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::support::{MOBILE, PASSWORD, cookie_value, json_body, request, set_cookie};

pub const WEB_ORIGIN: (&str, &str) = ("origin", "http://localhost:3000");

pub async fn sign_in(
    app: &Router,
    email: &str,
    password: &str,
    headers: &[(&str, &str)],
) -> Response {
    let body = json!({"email": email, "password": password});
    request(app.clone(), "POST", "/sessions", headers, Some(&body)).await
}

pub async fn session_exists(pool: &PgPool, session_id: Uuid) -> bool {
    sqlx::query_scalar("SELECT exists(SELECT 1 FROM sessions WHERE id = $1)")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .expect("an existence check")
}

pub async fn lifetime_matches(pool: &PgPool, session_id: Uuid, seconds: i32) -> bool {
    sqlx::query_scalar(
        "SELECT abs(extract(epoch FROM expires_at - now()) - $2) < 60 FROM sessions WHERE id = $1",
    )
    .bind(session_id)
    .bind(seconds)
    .fetch_one(pool)
    .await
    .expect("a session")
}

pub async fn revoked_ttl(session_id: Uuid) -> i64 {
    let url = std::env::var("REDIS_URL").expect("REDIS_URL, injected by infisical run");
    let client = redis::Client::open(url).expect("a redis url");
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .expect("a redis connection");
    redis::cmd("TTL")
        .arg(format!("auth:revoked:{session_id}"))
        .query_async(&mut connection)
        .await
        .expect("a ttl")
}

pub struct Session {
    pub access: String,
    pub refresh: String,
}

impl Session {
    pub fn id(&self) -> Uuid {
        let (id, _) = self.refresh.split_once('.').expect("<session>.<secret>");
        Uuid::parse_str(id).expect("a session id")
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Transport {
    Web,
    Mobile,
}

impl Transport {
    pub const BOTH: [Transport; 2] = [Transport::Web, Transport::Mobile];

    pub async fn sign_in(self, app: &Router, email: &str) -> Session {
        let headers: &[(&str, &str)] = match self {
            Transport::Web => &[],
            Transport::Mobile => &[MOBILE],
        };
        let response = sign_in(app, email, PASSWORD, headers).await;
        assert_eq!(response.status(), StatusCode::CREATED, "{self:?}");
        self.session_from(response).await
    }

    pub async fn session_from(self, response: Response) -> Session {
        match self {
            Transport::Web => Session {
                access: cookie_of(&response, "clinicore_access"),
                refresh: cookie_of(&response, "clinicore_refresh"),
            },
            Transport::Mobile => {
                let body = json_body(response).await;
                let tokens = &body["tokens"];
                Session {
                    access: tokens["accessToken"]
                        .as_str()
                        .expect("an access token")
                        .into(),
                    refresh: tokens["refreshToken"]
                        .as_str()
                        .expect("a refresh token")
                        .into(),
                }
            }
        }
    }

    pub async fn call(self, app: &Router, method: &str, path: &str, access: &str) -> Response {
        let cookie = format!("clinicore_access={access}");
        let bearer = format!("Bearer {access}");
        let headers = match self {
            Transport::Web => [WEB_ORIGIN, ("cookie", cookie.as_str())],
            Transport::Mobile => [MOBILE, ("authorization", bearer.as_str())],
        };
        request(app.clone(), method, path, &headers, None).await
    }

    pub async fn refresh(self, app: &Router, refresh: &str) -> Response {
        let path = "/sessions/current/tokens";
        match self {
            Transport::Web => {
                let cookie = format!("clinicore_refresh={refresh}");
                let headers = [WEB_ORIGIN, ("cookie", cookie.as_str())];
                request(app.clone(), "POST", path, &headers, None).await
            }
            Transport::Mobile => {
                let body = json!({"refreshToken": refresh});
                request(app.clone(), "POST", path, &[MOBILE], Some(&body)).await
            }
        }
    }
}

fn cookie_of(response: &Response, name: &str) -> String {
    let set_cookie = set_cookie(response, name).unwrap_or_else(|| panic!("a {name} cookie"));
    cookie_value(&set_cookie).to_string()
}

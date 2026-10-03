use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use super::{
    Transport, WEB_ORIGIN, lifetime_matches, revoke_without_deleting, revoked_ttl, session_exists,
};
use crate::support::{app_with, fresh_email, json_body, register, request, set_cookie};

#[sqlx::test(migrations = "../../migrations")]
async fn the_refresh_rotates_the_tokens_on_each_transport(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;

    for (transport, status, lifetime) in [
        (Transport::Web, StatusCode::NO_CONTENT, 86400),
        (Transport::Mobile, StatusCode::OK, 604800),
    ] {
        let session = transport.sign_in(&app, email).await;
        sqlx::query("UPDATE sessions SET expires_at = now() + interval '1 minute' WHERE id = $1")
            .bind(session.id())
            .execute(&pool)
            .await
            .expect("a session close to expiring");

        let response = transport.refresh(&app, &session.refresh).await;

        assert_eq!(response.status(), status, "{transport:?}");
        if let Transport::Mobile = transport {
            assert!(response.headers().get("set-cookie").is_none());
        }
        let rotated = transport.session_from(response).await;
        assert_eq!(rotated.id(), session.id());
        assert_ne!(rotated.refresh, session.refresh);
        assert!(
            lifetime_matches(&pool, session.id(), lifetime).await,
            "{transport:?}"
        );
        let current = transport
            .call(&app, "GET", "/sessions/current", &rotated.access)
            .await;
        assert_eq!(current.status(), StatusCode::OK);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn reusing_an_old_refresh_token_drops_the_whole_session(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;

    for transport in Transport::BOTH {
        let session = transport.sign_in(&app, email).await;
        let rotated = transport
            .session_from(transport.refresh(&app, &session.refresh).await)
            .await;

        let reused = transport.refresh(&app, &session.refresh).await;

        assert_eq!(reused.status(), StatusCode::UNAUTHORIZED, "{transport:?}");
        if let Transport::Web = transport {
            for name in ["clinicore_access", "clinicore_refresh"] {
                let cleared = set_cookie(&reused, name).expect("a cleared cookie");
                assert!(cleared.contains("Max-Age=0"), "{cleared}");
            }
        }
        assert_eq!(
            json_body(reused).await,
            json!({"type": "tag:clinicore.com.br,2026:session-reused", "title": "Refresh token reuse detected", "status": 401})
        );
        assert!(!session_exists(&pool, session.id()).await);
        let ttl = revoked_ttl(session.id()).await;
        assert!((604_700..=604_800).contains(&ttl), "{ttl}");
        let current = transport
            .call(&app, "GET", "/sessions/current", &rotated.access)
            .await;
        assert_eq!(current.status(), StatusCode::UNAUTHORIZED);
        let after = transport.refresh(&app, &rotated.refresh).await;
        assert_eq!(after.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            json_body(after).await["type"],
            "tag:clinicore.com.br,2026:invalid-session"
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_racing_refreshes_rotate_once_and_the_loser_drops_the_session(pool: PgPool) {
    let email = fresh_email();
    let email = email.as_str();
    let app = app_with(&pool);
    register(&app, &pool, email, true).await;
    let session = Transport::Web.sign_in(&app, email).await;
    let open_connections = [
        pool.acquire().await.expect("a connection"),
        pool.acquire().await.expect("a connection"),
    ];
    drop(open_connections);

    let (first, second) = tokio::join!(
        Transport::Web.refresh(&app, &session.refresh),
        Transport::Web.refresh(&app, &session.refresh),
    );

    let mut statuses = [first.status().as_u16(), second.status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, [204, 401]);
    assert!(!session_exists(&pool, session.id()).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_session_stops_renewing_thirty_days_after_it_opened(pool: PgPool) {
    let email = fresh_email();
    let app = app_with(&pool);
    register(&app, &pool, &email, true).await;

    for transport in Transport::BOTH {
        let session = transport.sign_in(&app, &email).await;
        sqlx::query(
            "UPDATE sessions SET created_at = now() - interval '30 days 1 minute' WHERE id = $1",
        )
        .bind(session.id())
        .execute(&pool)
        .await
        .expect("a session opened a month ago");

        let response = transport.refresh(&app, &session.refresh).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{transport:?}");
        assert_eq!(
            json_body(response).await["type"],
            "tag:clinicore.com.br,2026:invalid-session"
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_revoked_session_never_renews_even_when_its_row_survived(pool: PgPool) {
    let email = fresh_email();
    let app = app_with(&pool);
    register(&app, &pool, &email, true).await;

    for transport in Transport::BOTH {
        let session = transport.sign_in(&app, &email).await;
        revoke_without_deleting(session.id()).await;

        let response = transport.refresh(&app, &session.refresh).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{transport:?}");
        assert_eq!(
            json_body(response).await["type"],
            "tag:clinicore.com.br,2026:invalid-session"
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_refresh_token_renews_only_through_its_own_transport(pool: PgPool) {
    let email = fresh_email();
    let app = app_with(&pool);
    register(&app, &pool, &email, true).await;
    let session = Transport::Mobile.sign_in(&app, &email).await;
    let cookie = format!("clinicore_refresh={}", session.refresh);

    let crossed = request(
        app.clone(),
        "POST",
        "/sessions/current/tokens",
        &[WEB_ORIGIN, ("cookie", cookie.as_str())],
        None,
    )
    .await;

    assert_eq!(crossed.status(), StatusCode::UNAUTHORIZED);
    assert!(session_exists(&pool, session.id()).await);
    let own = Transport::Mobile.refresh(&app, &session.refresh).await;
    assert_eq!(own.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_session_refreshes_at_most_thirty_times_a_minute_whatever_the_address(pool: PgPool) {
    let email = fresh_email();
    let app = app_with(&pool);
    register(&app, &pool, &email, true).await;
    let mut session = Transport::Mobile.sign_in(&app, &email).await;

    for attempt in 1..=30 {
        let response = Transport::Mobile.refresh(&app, &session.refresh).await;
        assert_eq!(response.status(), StatusCode::OK, "attempt {attempt}");
        session = Transport::Mobile.session_from(response).await;
    }
    let response = Transport::Mobile.refresh(&app, &session.refresh).await;

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(session_exists(&pool, session.id()).await);
}

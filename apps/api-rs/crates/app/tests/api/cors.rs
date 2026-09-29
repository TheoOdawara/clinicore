use crate::support::{app, config_with, request};

#[tokio::test]
async fn cors_answers_the_allowed_origin_with_credentials_and_ignores_any_other() {
    let app = app(&config_with(&[]));

    let allowed = request(
        app.clone(),
        "GET",
        "/health",
        &[("origin", "http://localhost:3000")],
        None,
    )
    .await;
    let foreign = request(
        app,
        "GET",
        "/health",
        &[("origin", "http://evil.example")],
        None,
    )
    .await;

    assert_eq!(
        allowed.headers()["access-control-allow-origin"],
        "http://localhost:3000"
    );
    assert_eq!(
        allowed.headers()["access-control-allow-credentials"],
        "true"
    );
    assert!(
        foreign
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );
}

use axum::http::{Method, StatusCode};
use clinicore_core::config::AppEnv;

use crate::support::{app, send};

#[tokio::test]
async fn the_document_lists_the_routes_and_their_schemas_outside_production() {
    for app_env in [AppEnv::Development, AppEnv::Test] {
        let answer = send(app(app_env), Method::GET, "/api-json").await;

        assert_eq!(answer.status, StatusCode::OK);
        let document = answer.json();
        assert!(document["paths"]["/health"]["get"].is_object());
        assert!(document["paths"]["/users"]["post"].is_object());
        assert_eq!(
            document["components"]["schemas"]["SignUpRequest"]["required"],
            serde_json::json!(["name", "email", "password"])
        );
        assert_eq!(
            document["components"]["schemas"]["Problem"]["required"],
            serde_json::json!(["type", "title", "status"])
        );
    }
}

#[tokio::test]
async fn the_swagger_ui_is_served_outside_production() {
    let answer = send(app(AppEnv::Development), Method::GET, "/api/").await;

    assert_eq!(answer.status, StatusCode::OK);
    assert!(String::from_utf8_lossy(&answer.body).contains("swagger-ui"));
}

#[tokio::test]
async fn production_serves_neither_the_document_nor_the_ui() {
    for path in ["/api-json", "/api/"] {
        let answer = send(app(AppEnv::Production), Method::GET, path).await;

        assert_eq!(answer.status, StatusCode::NOT_FOUND, "{path}");
    }
}

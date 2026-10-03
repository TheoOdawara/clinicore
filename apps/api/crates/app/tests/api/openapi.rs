use crate::support::{config_with, content_type, json_body, send};
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn production_serves_neither_the_document_nor_the_ui() {
    let config = config_with(&[("APP_ENV", Some("production"))]);

    for path in ["/api-json", "/api"] {
        let response = send(crate::support::app(&config), "GET", path).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
        assert!(content_type(&response).starts_with("application/problem+json"));
        assert_eq!(
            json_body(response).await,
            json!({"type": "about:blank", "title": "Not Found", "status": 404})
        );
    }
}

#[tokio::test]
async fn development_serves_the_document_with_the_guards_on_every_business_route() {
    let config = config_with(&[("APP_ENV", Some("development"))]);

    let response = send(crate::support::app(&config), "GET", "/api-json").await;

    assert_eq!(response.status(), StatusCode::OK);
    let document = json_body(response).await;
    assert_eq!(document["info"]["title"], "Clinicore API");
    assert!(document["paths"]["/health"]["get"]["parameters"].is_null());
    let sign_up = &document["paths"]["/users"]["post"];
    assert_eq!(
        sign_up["parameters"][0]["schema"],
        json!({"type": "string", "enum": ["mobile"]})
    );
    assert_eq!(
        sign_up["responses"]["429"]["content"]["application/problem+json"]["schema"],
        json!({"$ref": "#/components/schemas/Problem"})
    );
    assert_eq!(
        document["components"]["schemas"]["SignUpRequest"]["required"],
        json!(["name", "email", "password"])
    );
}

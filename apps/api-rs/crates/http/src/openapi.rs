use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Clinicore API", version = "0.1.0"),
    paths(
        crate::health::check,
        crate::auth::sign_up,
        crate::auth::request_email_verification
    )
)]
struct ApiDoc;

pub fn docs() -> Router {
    SwaggerUi::new("/api")
        .url("/api-json", ApiDoc::openapi())
        .into()
}

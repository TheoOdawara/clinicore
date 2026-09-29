use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Clinicore API", version = "0.1.0"),
    paths(
        crate::health::check,
        crate::users::handlers::sign_up,
        crate::email_verifications::handlers::request_email_verification,
        crate::email_verifications::handlers::confirm_email,
        crate::sessions::handlers::sign_in,
        crate::sessions::handlers::current,
        crate::sessions::handlers::sign_out,
        crate::sessions::handlers::refresh
    )
)]
struct ApiDoc;

pub fn routes() -> Router {
    SwaggerUi::new("/api")
        .url("/api-json", ApiDoc::openapi())
        .into()
}

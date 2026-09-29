use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Empty {}

#[derive(Serialize, ToSchema)]
pub struct Health {
    status: &'static str,
    info: Empty,
    error: Empty,
    details: Empty,
}

#[utoipa::path(get, path = "/health", tag = "Health", responses((status = 200, body = Health)))]
pub async fn check() -> Json<Health> {
    Json(Health {
        status: "ok",
        info: Empty {},
        error: Empty {},
        details: Empty {},
    })
}

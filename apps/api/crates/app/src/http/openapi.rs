use utoipa::OpenApi;

use crate::http::error::Problem;

#[derive(OpenApi)]
#[openapi(info(title = "Clinicore API"), components(schemas(Problem)))]
pub struct ApiDoc;

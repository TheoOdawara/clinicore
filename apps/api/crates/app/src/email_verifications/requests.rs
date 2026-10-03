use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa::openapi::schema::Object;
use validator::Validate;

use crate::credentials::secret;
use crate::http::openapi;

static TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("^{}$", secret::PATTERN)).expect("a valid pattern"));

fn token_schema() -> Object {
    openapi::matching(&TOKEN)
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EmailVerificationRequest {
    #[serde(default)]
    #[schema(required = true)]
    #[validate(email, length(max = 320))]
    pub email: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EmailConfirmationRequest {
    #[serde(default)]
    #[validate(regex(path = *TOKEN))]
    #[schema(required = true, schema_with = token_schema)]
    pub token: String,
}

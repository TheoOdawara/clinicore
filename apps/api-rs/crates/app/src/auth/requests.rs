use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::http::validation;

#[derive(Deserialize, Validate, ToSchema)]
#[schema(as = SignUpDto)]
#[serde(deny_unknown_fields)]
pub struct SignUpRequest {
    #[serde(default, deserialize_with = "validation::trimmed")]
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[serde(default)]
    #[validate(email, length(max = 320))]
    pub email: String,
    #[serde(default)]
    #[validate(custom(function = "validation::strong_password"))]
    #[schema(min_length = 8, max_length = 128, example = "Clinica#2026")]
    pub password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[schema(as = EmailDto)]
#[serde(deny_unknown_fields)]
pub struct EmailRequest {
    #[serde(default)]
    #[validate(email, length(max = 320))]
    pub email: String,
}

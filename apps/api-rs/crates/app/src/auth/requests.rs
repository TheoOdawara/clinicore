use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::validation;

#[derive(Deserialize, Validate, ToSchema)]
#[schema(as = SignUpDto)]
pub struct SignUpRequest {
    #[serde(default, deserialize_with = "validation::trimmed")]
    #[validate(length(min = 1, max = 100, code = "LENGTH"))]
    pub name: String,
    #[serde(default)]
    #[validate(email(code = "IS_EMAIL"), length(max = 320, code = "LENGTH"))]
    pub email: String,
    #[serde(default)]
    #[validate(custom(function = "validation::strong_password"))]
    #[schema(min_length = 8, max_length = 128, example = "Clinica#2026")]
    pub password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[schema(as = EmailDto)]
pub struct EmailRequest {
    #[serde(default)]
    #[validate(email(code = "IS_EMAIL"), length(max = 320, code = "LENGTH"))]
    pub email: String,
}

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::credentials::{password, secret};

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PasswordResetRequest {
    #[serde(default)]
    #[schema(required = true)]
    #[validate(email, length(max = 320))]
    pub email: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PasswordResetConfirmationRequest {
    #[serde(default)]
    #[validate(regex(path = *secret::FORMAT))]
    #[schema(required = true, schema_with = secret::schema)]
    pub token: String,
    #[serde(default)]
    #[validate(custom(function = "password::strong_password"))]
    #[schema(
        required = true,
        min_length = 8,
        max_length = 128,
        example = "Clinica#2026"
    )]
    pub new_password: String,
}

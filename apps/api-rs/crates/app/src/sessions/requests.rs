use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use super::tokens::refresh;

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SignInRequest {
    #[serde(default)]
    #[schema(required = true)]
    #[validate(email, length(max = 320))]
    pub email: String,
    #[serde(default)]
    #[schema(required = true)]
    #[validate(length(min = 1, max = 128))]
    pub password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TokenRefreshRequest {
    #[serde(default)]
    #[validate(regex(path = *refresh::FORMAT))]
    #[schema(required = true, schema_with = refresh::schema)]
    pub refresh_token: String,
}

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::credentials::secret;

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
    #[validate(regex(path = *secret::FORMAT))]
    #[schema(required = true, schema_with = secret::schema)]
    pub token: String,
}

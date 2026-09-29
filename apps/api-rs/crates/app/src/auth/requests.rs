use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EmailVerificationRequest {
    #[serde(default)]
    #[validate(email, length(max = 320))]
    pub email: String,
}

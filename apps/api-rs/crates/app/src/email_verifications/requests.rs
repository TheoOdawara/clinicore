use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

static TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_-]{43}$").expect("a valid pattern"));

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
    #[schema(required = true, pattern = "^[A-Za-z0-9_-]{43}$")]
    pub token: String,
}

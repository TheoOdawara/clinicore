use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

static TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_-]{43}$").expect("a valid pattern"));
static REFRESH_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\.[A-Za-z0-9_-]{43}$")
        .expect("a valid pattern")
});

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EmailVerificationRequest {
    #[serde(default)]
    #[validate(email, length(max = 320))]
    pub email: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EmailConfirmationRequest {
    #[serde(default)]
    #[validate(regex(path = *TOKEN))]
    #[schema(pattern = "^[A-Za-z0-9_-]{43}$")]
    pub token: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SignInRequest {
    #[serde(default)]
    #[validate(email, length(max = 320))]
    pub email: String,
    #[serde(default)]
    #[validate(length(min = 1, max = 128))]
    pub password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TokenRefreshRequest {
    #[serde(default)]
    #[validate(regex(path = *REFRESH_TOKEN))]
    pub refresh_token: String,
}

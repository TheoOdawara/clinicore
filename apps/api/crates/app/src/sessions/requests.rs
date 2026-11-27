use utoipa::ToSchema;

use crate::credentials::email::Email;
use crate::http::validation::{Fields, Validate};
use crate::sessions::tokens::refresh::RefreshToken;

const PASSWORD_MAX_LENGTH: usize = 128;

#[derive(ToSchema)]
#[schema(example = json!({ "email": "ana@exemplo.com", "password": "Clinica#2026" }))]
pub struct SignInRequest {
    #[schema(value_type = String)]
    pub email: Email,
    #[schema(min_length = 1, max_length = 128)]
    pub password: String,
}

impl Validate for SignInRequest {
    fn validate(fields: &mut Fields) -> Option<Self> {
        let email = fields.field("email", Email::parse_field);
        let password = fields.field("password", parse_password);
        Some(Self {
            email: email?,
            password: password?,
        })
    }
}

#[derive(ToSchema)]
#[schema(example = json!({ "refreshToken": "<sessionId>.<43 base64url characters>" }))]
pub struct RefreshTokensRequest {
    #[schema(value_type = String, rename = "refreshToken")]
    pub refresh_token: RefreshToken,
}

impl Validate for RefreshTokensRequest {
    fn validate(fields: &mut Fields) -> Option<Self> {
        let refresh_token = fields.field("refreshToken", |raw| {
            RefreshToken::parse(&raw).ok_or("invalid")
        })?;
        Some(Self { refresh_token })
    }
}

fn parse_password(raw: String) -> Result<String, &'static str> {
    if (1..=PASSWORD_MAX_LENGTH).contains(&raw.chars().count()) {
        Ok(raw)
    } else {
        Err("length")
    }
}

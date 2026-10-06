use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

use crate::credentials::password;
use crate::http::validation;

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SignUpRequest {
    #[serde(default, deserialize_with = "validation::trimmed")]
    #[schema(required = true)]
    #[validate(length(min = 1, max = 100), custom(function = "person_name"))]
    pub name: String,
    #[serde(default)]
    #[schema(required = true)]
    #[validate(email, length(max = 320))]
    pub email: String,
    #[serde(default)]
    #[validate(custom(function = "password::strong_password"))]
    #[schema(
        required = true,
        min_length = 8,
        max_length = 128,
        example = "Clinica#2026"
    )]
    pub password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PasswordChangeRequest {
    #[serde(default)]
    #[schema(required = true)]
    #[validate(length(min = 1, max = 128))]
    pub current_password: String,
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

fn person_name(name: &str) -> Result<(), ValidationError> {
    static ALLOWED: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[\p{L}\p{M} .'-]+$").expect("a valid pattern"));
    static LETTER: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\p{L}").expect("a valid pattern"));

    if !ALLOWED.is_match(name) || !LETTER.is_match(name) {
        return Err(ValidationError::new("name"));
    }
    Ok(())
}

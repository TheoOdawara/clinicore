use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

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
    #[validate(custom(function = "strong_password"))]
    #[schema(
        required = true,
        min_length = 8,
        max_length = 128,
        example = "Clinica#2026"
    )]
    pub password: String,
}

fn strong_password(password: &str) -> Result<(), ValidationError> {
    static UPPERCASE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\p{Lu}").expect("a valid pattern"));
    static DIGIT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\p{Nd}").expect("a valid pattern"));
    static NEITHER_LETTER_NOR_DIGIT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[^\p{L}\p{Nd}]").expect("a valid pattern"));

    let is_strong = (8..=128).contains(&password.chars().count())
        && UPPERCASE.is_match(password)
        && DIGIT.is_match(password)
        && NEITHER_LETTER_NOR_DIGIT.is_match(password);
    if !is_strong {
        return Err(ValidationError::new("weak_password"));
    }
    Ok(())
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

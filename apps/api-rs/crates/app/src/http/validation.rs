use std::sync::LazyLock;

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use regex::Regex;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_path_to_error::Segment;
use validator::{Validate, ValidationError, ValidationErrors};

use super::error::{AppError, FieldError};

pub struct ValidJson<Body>(pub Body);

impl<State, Body> FromRequest<State> for ValidJson<Body>
where
    State: Send + Sync,
    Body: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &State) -> Result<Self, AppError> {
        let is_json = request
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("application/json"));
        if !is_json {
            return Err(AppError::Rejected(StatusCode::UNSUPPORTED_MEDIA_TYPE));
        }
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|rejection| AppError::Rejected(rejection.status()))?;

        let deserializer = &mut serde_json::Deserializer::from_slice(&bytes);
        let body: Body = serde_path_to_error::deserialize(deserializer).map_err(unreadable)?;
        body.validate()?;
        Ok(ValidJson(body))
    }
}

fn unreadable(error: serde_path_to_error::Error<serde_json::Error>) -> AppError {
    let names_a_field = matches!(error.path().iter().next(), Some(Segment::Map { .. }));
    if !error.inner().is_data() || !names_a_field {
        return AppError::Rejected(StatusCode::BAD_REQUEST);
    }
    AppError::Validation(vec![field_error(&error.path().to_string(), "invalid")])
}

impl From<ValidationErrors> for AppError {
    fn from(violations: ValidationErrors) -> Self {
        let mut errors: Vec<FieldError> = violations
            .field_errors()
            .into_iter()
            .filter_map(|(field, found)| {
                found
                    .first()
                    .map(|violation| field_error(&field, &violation.code))
            })
            .collect();
        errors.sort_by(|first, second| first.pointer.cmp(&second.pointer));
        AppError::Validation(errors)
    }
}

fn field_error(field: &str, code: &str) -> FieldError {
    FieldError {
        pointer: format!("#/{field}"),
        code: code.to_string(),
    }
}

pub fn trimmed<'de, Source: Deserializer<'de>>(source: Source) -> Result<String, Source::Error> {
    String::deserialize(source).map(|value| value.trim().to_string())
}

pub fn strong_password(password: &str) -> Result<(), ValidationError> {
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

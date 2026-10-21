use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_path_to_error::Segment;
use validator::{Validate, ValidationErrors};

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
                    .map(|violation| field_error(&wire_name(&field), &violation.code))
            })
            .collect();
        errors.sort_by(|first, second| first.pointer.cmp(&second.pointer));
        AppError::Validation(errors)
    }
}

fn wire_name(rust_field: &str) -> String {
    let mut words = rust_field.split('_');
    let mut name = words.next().unwrap_or_default().to_string();
    for word in words {
        let mut letters = word.chars();
        name.extend(letters.next().map(|first| first.to_ascii_uppercase()));
        name.push_str(letters.as_str());
    }
    name
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

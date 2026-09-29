use std::collections::BTreeMap;
use std::sync::LazyLock;

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::Response;
use regex::Regex;
use serde::de::{DeserializeOwned, IgnoredAny};
use serde::{Deserialize, Deserializer};
use serde_path_to_error::Segment;
use validator::{Validate, ValidationError};

use crate::error::{ErrorCode, FieldError, blank_problem, business_problem};

#[derive(Deserialize)]
struct Envelope<Body> {
    #[serde(flatten)]
    known: Body,
    #[serde(flatten)]
    unknown: BTreeMap<String, IgnoredAny>,
}

pub struct ValidJson<Body>(pub Body);

impl<State, Body> FromRequest<State> for ValidJson<Body>
where
    State: Send + Sync,
    Body: DeserializeOwned + Validate,
{
    type Rejection = Response;

    async fn from_request(request: Request, state: &State) -> Result<Self, Response> {
        let is_json = request
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("application/json"));
        if !is_json {
            return Err(blank_problem(StatusCode::UNSUPPORTED_MEDIA_TYPE));
        }
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(|rejection| blank_problem(rejection.status()))?;

        let envelope: Envelope<Body> = match serde_json::from_slice(&bytes) {
            Ok(envelope) => envelope,
            Err(error) if error.is_data() => return Err(mistyped::<Body>(&bytes)),
            Err(_) => return Err(blank_problem(StatusCode::BAD_REQUEST)),
        };

        let mut errors: Vec<FieldError> = envelope
            .unknown
            .keys()
            .map(|field| field_error(field, "WHITELIST_VALIDATION"))
            .collect();
        if let Err(violations) = envelope.known.validate() {
            let mut fields: Vec<FieldError> = violations
                .field_errors()
                .into_iter()
                .filter_map(|(field, found)| {
                    found
                        .first()
                        .map(|violation| field_error(&field, &violation.code))
                })
                .collect();
            fields.sort_by(|first, second| first.pointer.cmp(&second.pointer));
            errors.extend(fields);
        }
        if !errors.is_empty() {
            return Err(invalid(errors));
        }
        Ok(ValidJson(envelope.known))
    }
}

fn mistyped<Body: DeserializeOwned>(bytes: &[u8]) -> Response {
    let deserializer = &mut serde_json::Deserializer::from_slice(bytes);
    let Err(error) = serde_path_to_error::deserialize::<_, Body>(deserializer) else {
        return blank_problem(StatusCode::BAD_REQUEST);
    };
    let Some(Segment::Map { .. }) = error.path().iter().next() else {
        return blank_problem(StatusCode::BAD_REQUEST);
    };
    invalid(vec![field_error(&error.path().to_string(), "IS_STRING")])
}

fn invalid(errors: Vec<FieldError>) -> Response {
    business_problem(ErrorCode::ValidationFailed, errors)
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
        return Err(ValidationError::new("WEAK_PASSWORD"));
    }
    Ok(())
}

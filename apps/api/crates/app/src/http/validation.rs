use axum::Json;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use serde_json::{Map, Value};

use crate::http::error::{AppError, FieldError};

const INVALID: &str = "invalid";

pub trait Validate: Sized {
    fn validate(fields: &mut Fields) -> Option<Self>;
}

pub struct Fields {
    values: Map<String, Value>,
    errors: Vec<FieldError>,
}

impl Fields {
    pub fn field<T>(
        &mut self,
        name: &'static str,
        parse: impl FnOnce(String) -> Result<T, &'static str>,
    ) -> Option<T> {
        let Some(Value::String(raw)) = self.values.remove(name) else {
            self.refuse(name, INVALID);
            return None;
        };
        parse(raw).map_err(|code| self.refuse(name, code)).ok()
    }

    fn refuse(&mut self, name: &str, code: &'static str) {
        self.errors.push(FieldError {
            pointer: format!("#/{name}"),
            code,
        });
    }
}

pub struct ValidJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidJson<T>
where
    S: Send + Sync,
    T: Validate,
{
    type Rejection = AppError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(values) = Json::<Map<String, Value>>::from_request(request, state)
            .await
            .map_err(|rejection| match rejection.status() {
                StatusCode::PAYLOAD_TOO_LARGE => AppError::PayloadTooLarge,
                StatusCode::UNSUPPORTED_MEDIA_TYPE => AppError::UnsupportedMediaType,
                _ => AppError::MalformedBody,
            })?;
        let mut fields = Fields {
            values,
            errors: Vec::new(),
        };
        let validated = T::validate(&mut fields);
        let unknown: Vec<String> = fields.values.keys().cloned().collect();
        for name in unknown {
            fields.refuse(&name, INVALID);
        }
        match validated {
            Some(value) if fields.errors.is_empty() => Ok(Self(value)),
            _ => {
                assert!(
                    !fields.errors.is_empty(),
                    "a request that validates to nothing names the fields it refused"
                );
                Err(AppError::Validation(fields.errors))
            }
        }
    }
}

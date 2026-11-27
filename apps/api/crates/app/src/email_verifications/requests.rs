use utoipa::ToSchema;

use crate::credentials::email::Email;
use crate::credentials::secret::Secret;
use crate::http::validation::{Fields, Validate};

#[derive(ToSchema)]
#[schema(example = json!({ "email": "ana@exemplo.com" }))]
pub struct EmailVerificationRequest {
    #[schema(value_type = String)]
    pub email: Email,
}

impl Validate for EmailVerificationRequest {
    fn validate(fields: &mut Fields) -> Option<Self> {
        let email = fields.field("email", Email::parse_field)?;
        Some(Self { email })
    }
}

#[derive(ToSchema)]
#[schema(example = json!({ "token": "q6m0o4RzW0n3bGJ8m4v0n1Xk2d5c7a9e1f3h5j7l9n0" }))]
pub struct EmailVerificationConfirmationRequest {
    #[schema(value_type = String, min_length = 43, max_length = 43)]
    pub token: Secret,
}

impl Validate for EmailVerificationConfirmationRequest {
    fn validate(fields: &mut Fields) -> Option<Self> {
        let token = fields.field("token", |raw| Secret::parse(&raw).ok_or("invalid"))?;
        Some(Self { token })
    }
}

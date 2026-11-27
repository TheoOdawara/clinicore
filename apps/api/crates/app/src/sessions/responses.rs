use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::sessions::tokens::access::ACCESS_LIFETIME;
use crate::sessions::tokens::refresh::RefreshToken;

#[derive(Serialize, ToSchema)]
pub struct SessionResponse {
    pub user: UserResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokensResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct RefreshTokensResponse {
    pub tokens: TokensResponse,
}

#[derive(Serialize, ToSchema)]
#[schema(example = json!({ "id": "0b8f2c1e-5a4d-4c3b-9e7f-1a2b3c4d5e6f", "name": "Ana Souza", "email": "ana@exemplo.com" }))]
pub struct UserResponse {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokensResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: u64,
}

impl TokensResponse {
    pub fn new(access_token: String, refresh_token: &RefreshToken) -> Self {
        Self {
            access_token,
            refresh_token: refresh_token.to_string(),
            access_token_expires_in: ACCESS_LIFETIME.as_secs(),
        }
    }
}

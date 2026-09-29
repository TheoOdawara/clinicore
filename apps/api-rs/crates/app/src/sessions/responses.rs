use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_in: u64,
}

#[derive(Serialize, ToSchema)]
pub struct SessionResponse {
    pub user: SessionUser,
}

#[derive(Serialize, ToSchema)]
pub struct SignInResponse {
    pub user: SessionUser,
    pub tokens: SessionTokens,
}

#[derive(Serialize, ToSchema)]
pub struct TokenRefreshResponse {
    pub tokens: SessionTokens,
}

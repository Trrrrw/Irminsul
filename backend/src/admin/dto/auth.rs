use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

use crate::admin::dto::users::AdminUserView;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub invitation_token: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: AdminUserView,
    pub csrf_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CsrfTokenResponse {
    pub csrf_token: String,
}

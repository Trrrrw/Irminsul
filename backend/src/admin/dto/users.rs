use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

use crate::admin::{
    entities::users,
    model::{AdminRole, AdminUserStatus},
};

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserView {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub role: AdminRole,
    pub status: AdminUserStatus,
    pub must_change_password: bool,
    pub must_change_username: bool,
    pub must_set_email: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSelfProfileRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetUserStatusRequest {
    pub enabled: bool,
}

impl From<users::Model> for AdminUserView {
    fn from(value: users::Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            role: value.role,
            status: value.status,
            must_change_password: value.must_change_password,
            must_change_username: value.must_change_username,
            must_set_email: value.must_set_email,
        }
    }
}

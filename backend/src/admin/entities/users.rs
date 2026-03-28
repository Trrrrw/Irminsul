use sea_orm::entity::prelude::*;

use crate::admin::model::{AdminRole, AdminUserStatus};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ADMIN_USERS")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(unique)]
    pub username: String,
    #[sea_orm(unique)]
    pub email: Option<String>,
    pub password_hash: String,
    pub role: AdminRole,
    pub status: AdminUserStatus,
    pub must_change_password: bool,
    pub must_change_username: bool,
    pub must_set_email: bool,
    pub last_login_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ActiveModelBehavior for ActiveModel {}

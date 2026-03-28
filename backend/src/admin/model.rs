use salvo::oapi::ToSchema;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 管理后台用户角色。
#[derive(
    EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum AdminRole {
    #[sea_orm(string_value = "owner")]
    Owner,
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "editor")]
    Editor,
    #[sea_orm(string_value = "viewer")]
    Viewer,
}

/// 管理后台账号状态。
#[derive(
    EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum AdminUserStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "disabled")]
    Disabled,
}

/// 邀请码生命周期状态。
#[derive(
    EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum InvitationStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "consumed")]
    Consumed,
    #[sea_orm(string_value = "revoked")]
    Revoked,
    #[sea_orm(string_value = "expired")]
    Expired,
}

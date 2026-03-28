use sea_orm::entity::prelude::*;

use crate::admin::model::{AdminRole, InvitationStatus};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ADMIN_INVITATIONS")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    #[sea_orm(unique)]
    pub token_hash: String,
    pub role: AdminRole,
    pub status: InvitationStatus,
    pub invited_email: Option<String>,
    pub note: Option<String>,
    pub created_by_user_id: i64,
    pub created_at: i64,
    pub expires_at: i64,
    pub consumed_at: Option<i64>,
    pub consumed_by_user_id: Option<i64>,
}

impl ActiveModelBehavior for ActiveModel {}

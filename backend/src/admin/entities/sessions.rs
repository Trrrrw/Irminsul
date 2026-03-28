use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ADMIN_SESSIONS")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub admin_user_id: i64,
    #[sea_orm(unique)]
    pub token_hash: String,
    #[sea_orm(unique)]
    pub csrf_token_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub expires_at: i64,
    pub last_seen_at: i64,
    pub revoked_at: Option<i64>,
    pub created_ip: Option<String>,
    pub last_seen_ip: Option<String>,
    pub user_agent: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}

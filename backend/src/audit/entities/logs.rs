use sea_orm::entity::prelude::*;

use crate::audit::model::AuditActorType;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "AUDIT_LOGS")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub actor_type: AuditActorType,
    pub actor_user_id: Option<i64>,
    pub actor_label: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub summary: String,
    pub metadata_json: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: i64,
}

impl ActiveModelBehavior for ActiveModel {}

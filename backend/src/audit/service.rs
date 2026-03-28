use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde_json::Value;

use crate::audit::{entities::logs, model::AuditActorType};

/// 审计日志写入参数。
#[derive(Debug, Clone)]
pub struct AuditLogParams {
    pub actor_type: AuditActorType,
    pub actor_user_id: Option<i64>,
    pub actor_label: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub summary: String,
    pub metadata: Option<Value>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

pub async fn write_audit_log(params: AuditLogParams) {
    let log = logs::ActiveModel {
        actor_type: Set(params.actor_type),
        actor_user_id: Set(params.actor_user_id),
        actor_label: Set(params.actor_label),
        action: Set(params.action),
        target_type: Set(params.target_type),
        target_id: Set(params.target_id),
        summary: Set(params.summary),
        metadata_json: Set(params.metadata.map(|value| value.to_string())),
        ip: Set(params.ip),
        user_agent: Set(params.user_agent),
        created_at: Set(crate::admin::middlewares::auth::unix_timestamp()),
        ..Default::default()
    };
    let _ = log.insert(crate::db::pool()).await;
}

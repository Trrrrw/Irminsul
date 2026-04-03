use sea_orm::Value as SeaValue;
use serde_json::Value;

use crate::{
    admin::{db, entities::audit_logs, repository},
    audit::model::AuditActorType,
};

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

/// 写入一条审计日志。
pub async fn write_audit_log(params: AuditLogParams) {
    let _ = repository::execute(
        db::database(),
        "INSERT INTO AUDIT_LOGS
         (actor_type, actor_user_id, actor_label, action, target_type, target_id, summary, metadata_json, ip, user_agent, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        vec![
            SeaValue::from(params.actor_type.as_str()),
            SeaValue::from(params.actor_user_id),
            SeaValue::from(params.actor_label),
            SeaValue::from(params.action),
            SeaValue::from(params.target_type),
            SeaValue::from(params.target_id),
            SeaValue::from(params.summary),
            SeaValue::from(params.metadata.map(|value| value.to_string())),
            SeaValue::from(params.ip),
            SeaValue::from(params.user_agent),
            SeaValue::from(crate::admin::middlewares::auth::unix_timestamp()),
        ],
    )
    .await;
}

/// 列出后台审计日志。
pub async fn list_audit_logs() -> Result<Vec<audit_logs::Model>, String> {
    let rows = repository::query_all(
        db::database(),
        "SELECT id, actor_type, actor_user_id, actor_label, action, target_type, target_id, summary, metadata_json, ip, user_agent, created_at
         FROM AUDIT_LOGS ORDER BY id DESC",
        Vec::new(),
    )
    .await
    .map_err(|error| format!("查询审计日志失败: {error}"))?;

    rows.iter()
        .map(repository::map_audit_log_row)
        .collect::<Result<Vec<_>, _>>()
}

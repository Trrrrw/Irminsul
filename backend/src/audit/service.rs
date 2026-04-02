use serde_json::Value;

use crate::{
    admin::{db, entities::audit_logs},
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
    let Ok(conn) = db::database().connect() else {
        return;
    };

    let _ = conn
        .execute(
            "INSERT INTO AUDIT_LOGS
             (actor_type, actor_user_id, actor_label, action, target_type, target_id, summary, metadata_json, ip, user_agent, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            turso::params![
                params.actor_type.as_str(),
                params.actor_user_id,
                params.actor_label,
                params.action,
                params.target_type,
                params.target_id,
                params.summary,
                params.metadata.map(|value| value.to_string()),
                params.ip,
                params.user_agent,
                crate::admin::middlewares::auth::unix_timestamp(),
            ],
        )
        .await;
}

/// 列出后台审计日志。
pub async fn list_audit_logs() -> Result<Vec<audit_logs::Model>, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开后台数据库失败: {error}"))?;
    let mut rows = conn
        .query(
            "SELECT id, actor_type, actor_user_id, actor_label, action, target_type, target_id, summary, metadata_json, ip, user_agent, created_at
             FROM AUDIT_LOGS ORDER BY id DESC",
            (),
        )
        .await
        .map_err(|error| format!("查询审计日志失败: {error}"))?;

    let mut values = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取审计日志失败: {error}"))?
    {
        values.push(audit_logs::Model {
            id: row.get(0).map_err(|error| error.to_string())?,
            actor_type: row
                .get::<String>(1)
                .map_err(|error| error.to_string())?
                .parse::<AuditActorType>()
                .map_err(|error| error.to_string())?,
            actor_user_id: row.get(2).map_err(|error| error.to_string())?,
            actor_label: row.get(3).map_err(|error| error.to_string())?,
            action: row.get(4).map_err(|error| error.to_string())?,
            target_type: row.get(5).map_err(|error| error.to_string())?,
            target_id: row.get(6).map_err(|error| error.to_string())?,
            summary: row.get(7).map_err(|error| error.to_string())?,
            metadata_json: row.get(8).map_err(|error| error.to_string())?,
            ip: row.get(9).map_err(|error| error.to_string())?,
            user_agent: row.get(10).map_err(|error| error.to_string())?,
            created_at: row.get(11).map_err(|error| error.to_string())?,
        });
    }

    Ok(values)
}

use crate::audit::{
    model::AuditActorType,
    service::{AuditLogParams, write_audit_log as write_general_audit_log},
};

pub async fn write_audit_log(
    actor_user_id: Option<i64>,
    actor_label: Option<String>,
    action: &str,
    target_type: &str,
    target_id: Option<String>,
    summary: &str,
    metadata: Option<serde_json::Value>,
    ip: Option<String>,
    user_agent: Option<String>,
) {
    write_general_audit_log(AuditLogParams {
        actor_type: AuditActorType::AdminUser,
        actor_user_id,
        actor_label,
        action: action.to_string(),
        target_type: target_type.to_string(),
        target_id,
        summary: summary.to_string(),
        metadata,
        ip,
        user_agent,
    })
    .await;
}

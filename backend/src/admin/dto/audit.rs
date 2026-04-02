use salvo::oapi::ToSchema;
use serde::Serialize;

/// 审计日志视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct AuditLogView {
    pub id: i64,
    pub actor_type: String,
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

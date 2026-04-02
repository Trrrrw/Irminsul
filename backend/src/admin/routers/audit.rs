use salvo::{Router, prelude::*};

use crate::admin::{
    dto::audit::AuditLogView,
    middlewares::{
        auth::{require_authenticated_admin, require_completed_profile},
        origin::require_same_origin,
    },
};

pub fn router() -> Router {
    Router::with_path("api/admin/audit")
        .hoop(require_same_origin)
        .hoop(require_authenticated_admin)
        .hoop(require_completed_profile)
        .push(Router::with_path("logs").get(list_logs))
}

/// 获取后台审计日志列表
#[endpoint(
    tags("admin.audit"),
    responses((status_code = 200, description = "获取审计日志成功", body = Vec<AuditLogView>))
)]
async fn list_logs(res: &mut Response) {
    let values = crate::audit::service::list_audit_logs()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|log| AuditLogView {
            id: log.id,
            actor_type: log.actor_type.as_str().to_string(),
            actor_user_id: log.actor_user_id,
            actor_label: log.actor_label,
            action: log.action,
            target_type: log.target_type,
            target_id: log.target_id,
            summary: log.summary,
            metadata_json: log.metadata_json,
            ip: log.ip,
            user_agent: log.user_agent,
            created_at: log.created_at,
        })
        .collect::<Vec<_>>();
    res.render(Json(values));
}

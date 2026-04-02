use salvo::{Router, prelude::*};

use crate::{
    admin::{
        errors::render_api_error,
        middlewares::{
            auth::{
                get_current_admin, require_authenticated_admin, require_completed_profile,
                require_role,
            },
            csrf::require_csrf,
            origin::require_same_origin,
        },
        model::AdminRole,
        services::audit::write_audit_log,
    },
    content::dto::content::EmbeddingJobView,
    vector::jobs,
};

pub fn router() -> Router {
    Router::with_path("api/admin/embedding")
        .hoop(require_same_origin)
        .hoop(require_authenticated_admin)
        .hoop(require_completed_profile)
        .push(Router::with_path("jobs").get(list_jobs))
        .push(
            Router::with_path("jobs/<id>/retry")
                .hoop(require_csrf)
                .post(retry_job),
        )
}

/// 获取向量任务列表
#[endpoint(
    tags("admin.embedding"),
    responses((status_code = 200, description = "获取向量任务列表成功", body = Vec<EmbeddingJobView>))
)]
async fn list_jobs(req: &mut Request, res: &mut Response) {
    let content_item_id = req.query::<i64>("content_item_id");
    match jobs::list_jobs(content_item_id).await {
        Ok(values) => res.render(Json(values)),
        Err(error) => render_api_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            "embedding_jobs_list_failed",
            error,
        ),
    }
}

/// 手动重试向量任务
#[endpoint(
    tags("admin.embedding"),
    responses((status_code = 200, description = "重试向量任务成功"))
)]
async fn retry_job(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(res, StatusCode::BAD_REQUEST, "invalid_id", "invalid job id");
        return;
    };
    let actor = get_current_admin(depot).cloned();
    match jobs::retry_job(
        id,
        actor.as_ref().map(|value| value.id),
        actor.as_ref().map(|value| value.username.clone()),
    )
    .await
    {
        Ok(new_job_id) => {
            audit_embedding_change(
                req,
                depot,
                "retry_embedding_job",
                "embedding_job",
                id,
                "retried embedding job",
            )
            .await;
            res.render(Json(serde_json::json!({ "job_id": new_job_id })));
        }
        Err(error) if error == "embedding_job_not_found" => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "embedding_job_not_found",
            "embedding job not found",
        ),
        Err(error) => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "embedding_retry_failed",
            error,
        ),
    }
}

async fn audit_embedding_change(
    req: &Request,
    depot: &Depot,
    action: &str,
    target_type: &str,
    target_id: i64,
    summary: &str,
) {
    if let Some(actor) = get_current_admin(depot) {
        write_audit_log(
            Some(actor.id),
            Some(actor.username.clone()),
            action,
            target_type,
            Some(target_id.to_string()),
            summary,
            None,
            crate::admin::services::auth::client_ip(req),
            crate::admin::services::auth::user_agent(req),
        )
        .await;
    }
}

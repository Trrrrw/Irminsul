use salvo::{Router, prelude::*};

use crate::admin::{
    dto::embedding::{AdminSettingsView, UpdateAdminSettingsRequest},
    errors::render_api_error,
    middlewares::{
        auth::{
            get_current_admin, require_authenticated_admin, require_completed_profile, require_role,
        },
        csrf::require_csrf,
        origin::require_same_origin,
    },
    model::AdminRole,
    services::{audit::write_audit_log, embedding},
};
use crate::vector::jobs;

/// panel 设置页相关接口
pub fn router() -> Router {
    Router::with_path("api/admin/settings")
        .hoop(require_same_origin)
        .hoop(require_authenticated_admin)
        .hoop(require_completed_profile)
        .get(get_settings)
        .push(Router::new().hoop(require_csrf).patch(update_settings))
}

/// 获取当前设置
#[endpoint(
    tags("admin.settings"),
    responses((status_code = 200, description = "获取设置成功", body = AdminSettingsView))
)]
async fn get_settings(res: &mut Response) {
    match embedding::get_settings_bundle().await {
        Ok(settings) => res.render(Json(settings)),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 更新当前设置
#[endpoint(
    tags("admin.settings"),
    request_body = UpdateAdminSettingsRequest,
    responses((status_code = 200, description = "更新设置成功", body = AdminSettingsView))
)]
async fn update_settings(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(actor) = get_current_admin(depot).cloned() else {
        render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "authentication required",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpdateAdminSettingsRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };

    if let Some(embedding_payload) = payload.embedding {
        if let Some(provider_payloads) = embedding_payload.providers {
            for provider_payload in provider_payloads {
                let resource_id = provider_payload.id.unwrap_or_default();
                match embedding::upsert_provider(provider_payload).await {
                    Ok(provider) => {
                        let action = if resource_id == 0 {
                            "create_embedding_provider"
                        } else {
                            "update_embedding_provider"
                        };
                        let summary = if resource_id == 0 {
                            "created embedding provider from settings"
                        } else {
                            "updated embedding provider from settings"
                        };
                        audit_embedding_change(
                            req,
                            depot,
                            action,
                            "embedding_provider",
                            provider.id,
                            summary,
                        )
                        .await;
                    }
                    Err("embedding_provider_not_found") => {
                        render_api_error(
                            res,
                            StatusCode::NOT_FOUND,
                            "embedding_provider_not_found",
                            "embedding provider not found",
                        );
                        return;
                    }
                    Err(error) => {
                        render_api_error(res, StatusCode::BAD_REQUEST, error, error);
                        return;
                    }
                }
            }
        }

        if let Some(api_key_payloads) = embedding_payload.api_keys {
            for api_key_payload in api_key_payloads {
                let resource_id = api_key_payload.id.unwrap_or_default();
                match embedding::upsert_api_key(api_key_payload).await {
                    Ok(api_key) => {
                        let action = if resource_id == 0 {
                            "create_embedding_api_key"
                        } else {
                            "update_embedding_api_key"
                        };
                        let summary = if resource_id == 0 {
                            "created embedding api key from settings"
                        } else {
                            "updated embedding api key from settings"
                        };
                        audit_embedding_change(
                            req,
                            depot,
                            action,
                            "embedding_api_key",
                            api_key.id,
                            summary,
                        )
                        .await;
                    }
                    Err("embedding_provider_not_found") => {
                        render_api_error(
                            res,
                            StatusCode::NOT_FOUND,
                            "embedding_provider_not_found",
                            "embedding provider not found",
                        );
                        return;
                    }
                    Err("embedding_api_key_not_found") => {
                        render_api_error(
                            res,
                            StatusCode::NOT_FOUND,
                            "embedding_api_key_not_found",
                            "embedding api key not found",
                        );
                        return;
                    }
                    Err(error) => {
                        render_api_error(res, StatusCode::BAD_REQUEST, error, error);
                        return;
                    }
                }
            }
        }

        if let Some(settings_payload) = embedding_payload.settings {
            match embedding::update_settings(
                settings_payload,
                actor.id,
                actor.username.clone(),
                crate::admin::services::auth::client_ip(req),
                crate::admin::services::auth::user_agent(req),
            )
            .await
            {
                Ok((_, changed_model)) => {
                    if changed_model {
                        match jobs::enqueue_full_rebuild(
                            Some(actor.id),
                            Some(actor.username.clone()),
                        )
                        .await
                        {
                            Ok(enqueued) => {
                                write_audit_log(
                                    Some(actor.id),
                                    Some(actor.username.clone()),
                                    "enqueue_full_embedding_rebuild",
                                    "embedding_jobs",
                                    None,
                                    "triggered full embedding rebuild after model change",
                                    Some(serde_json::json!({ "enqueued": enqueued })),
                                    crate::admin::services::auth::client_ip(req),
                                    crate::admin::services::auth::user_agent(req),
                                )
                                .await;
                            }
                            Err(error) => {
                                render_api_error(
                                    res,
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    "embedding_rebuild_enqueue_failed",
                                    error,
                                );
                                return;
                            }
                        }
                    }
                }
                Err("embedding_provider_not_found") => {
                    render_api_error(
                        res,
                        StatusCode::NOT_FOUND,
                        "embedding_provider_not_found",
                        "embedding provider not found",
                    );
                    return;
                }
                Err(error) => {
                    render_api_error(res, StatusCode::BAD_REQUEST, error, error);
                    return;
                }
            }
        }
    }

    match embedding::get_settings_bundle().await {
        Ok(settings) => res.render(Json(settings)),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
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

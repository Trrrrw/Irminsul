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
    services::embedding,
};

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
    responses((status_code = 200, description = "更新设置成功"))
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

    match embedding::update_settings(
        payload,
        actor.id,
        actor.username.clone(),
        crate::admin::services::auth::client_ip(req),
        crate::admin::services::auth::user_agent(req),
    )
    .await
    {
        Ok(result) => res.render(Json(serde_json::json!({
            "settings": result.settings,
            "meta": {
                "changed_model": result.changed_model,
                "changed_fields": result.changed_fields,
            }
        }))),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

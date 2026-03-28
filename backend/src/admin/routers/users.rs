use salvo::{Router, prelude::*};

use crate::admin::{
    dto::users::{SetUserStatusRequest, UpdateSelfProfileRequest},
    errors::render_api_error,
    middlewares::{
        auth::{
            get_current_admin, require_authenticated_admin, require_completed_profile, require_role,
        },
        csrf::require_csrf,
        origin::require_same_origin,
    },
    model::AdminRole,
    services::{
        audit::write_audit_log,
        auth::{client_ip, user_agent},
        users::{list_users, set_user_enabled, update_self_profile},
    },
};

pub fn router() -> Router {
    Router::with_path("api/admin/users")
        .hoop(require_same_origin)
        .hoop(require_authenticated_admin)
        .hoop(require_completed_profile)
        .push(Router::new().get(list))
        .push(Router::with_path("me").hoop(require_csrf).patch(update_me))
        .push(
            Router::with_path("<id>/status")
                .hoop(require_csrf)
                .patch(set_status),
        )
}

/// 获取管理员列表。
///
/// 仅 `owner` 可调用。用于后台展示管理员账号及其当前状态。
#[endpoint(
    tags("admin.users"),
    responses(
        (status_code = 200, description = "获取管理员列表成功", body = Vec<crate::admin::dto::users::AdminUserView>),
        (status_code = 403, description = "需要 owner 权限")
    )
)]
async fn list(depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    res.render(Json(list_users().await));
}

/// 更新当前管理员自己的资料。
///
/// 该接口同时承担首次登录后的资料完善流程：
/// - 首个 `owner` 首次登录后必须补邮箱
/// - 通过邀请码创建的用户首次登录后必须修改用户名和密码
///
/// 普通已登录用户修改密码时需要提供 `current_password`。
#[endpoint(
    tags("admin.users"),
    request_body = UpdateSelfProfileRequest,
    responses(
        (status_code = 200, description = "更新自己的资料成功", body = crate::admin::dto::users::AdminUserView),
        (status_code = 400, description = "请求无效，或仍需补全资料"),
        (status_code = 403, description = "当前密码错误"),
        (status_code = 409, description = "更新失败，例如唯一约束冲突")
    )
)]
async fn update_me(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let Some(current_admin) = get_current_admin(depot) else {
        render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "authentication required",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpdateSelfProfileRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };

    match update_self_profile(current_admin.id, payload).await {
        Ok(user) => {
            write_audit_log(
                Some(current_admin.id),
                Some(current_admin.username.clone()),
                "update_self_profile",
                "admin_user",
                Some(user.id.to_string()),
                "updated own profile",
                None,
                client_ip(req),
                user_agent(req),
            )
            .await;
            res.render(Json(crate::admin::dto::users::AdminUserView::from(user)));
        }
        Err("username_change_required") => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "username_change_required",
            "username change is required",
        ),
        Err("username_required") => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "username_required",
            "username is required",
        ),
        Err("password_change_required") => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "password_change_required",
            "password change is required",
        ),
        Err("email_required") | Err("owner_email_required") => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "email_required",
            "email is required",
        ),
        Err("current_password_required") => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "current_password_required",
            "current password is required",
        ),
        Err("current_password_invalid") => render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "current_password_invalid",
            "current password is invalid",
        ),
        Err("password_policy_failed") => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "password_policy_failed",
            "password does not satisfy the policy",
        ),
        Err(_) => render_api_error(
            res,
            StatusCode::CONFLICT,
            "profile_update_failed",
            "failed to update profile",
        ),
    }
}

/// 启用或禁用指定管理员账号。
///
/// 仅 `owner` 可调用。当前实现禁止：
/// - 禁用自己
/// - 禁用系统中的最后一个 `owner`
#[endpoint(
    tags("admin.users"),
    request_body = SetUserStatusRequest,
    responses(
        (status_code = 200, description = "更新用户状态成功", body = crate::admin::dto::users::AdminUserView),
        (status_code = 400, description = "用户 ID 无效"),
        (status_code = 403, description = "权限不足，或尝试禁用自己/最后一个 owner"),
        (status_code = 404, description = "用户不存在")
    )
)]
async fn set_status(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let Some(current_admin) = get_current_admin(depot) else {
        render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "authentication required",
        );
        return;
    };
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }

    let Some(target_user_id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid user id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<SetUserStatusRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };

    match set_user_enabled(
        target_user_id,
        payload.enabled,
        current_admin.id,
        &current_admin.role,
        client_ip(req),
        user_agent(req),
    )
    .await
    {
        Ok(user) => res.render(Json(crate::admin::dto::users::AdminUserView::from(user))),
        Err("user_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "user_not_found",
            "user does not exist",
        ),
        Err("owner_management_forbidden") => render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_management_forbidden",
            "cannot manage owner user with current role",
        ),
        Err("cannot_disable_self") => render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "cannot_disable_self",
            "cannot disable the current user",
        ),
        Err("cannot_disable_last_owner") => render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "cannot_disable_last_owner",
            "cannot disable the last owner",
        ),
        Err(_) => render_api_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            "user_status_update_failed",
            "failed to update user status",
        ),
    }
}

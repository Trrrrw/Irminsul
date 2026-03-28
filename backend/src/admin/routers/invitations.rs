use salvo::{Router, prelude::*};

use crate::admin::{
    dto::invitations::{CreateInvitationRequest, CreateInvitationResponse},
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
        auth::{client_ip, user_agent},
        invitations::{create_invitation, list_invitations, revoke_invitation},
    },
};

pub fn router() -> Router {
    Router::with_path("api/admin/invitations")
        .hoop(require_same_origin)
        .hoop(require_authenticated_admin)
        .hoop(require_completed_profile)
        .push(Router::new().get(list))
        .push(Router::new().hoop(require_csrf).post(create))
        .push(Router::with_path("<id>").hoop(require_csrf).delete(revoke))
}

/// 获取邀请码列表。
///
/// 仅 `owner` 可调用。返回当前后台邀请码的状态列表，便于面板展示和管理。
#[endpoint(
    tags("admin.invitations"),
    responses(
        (status_code = 200, description = "获取邀请码列表成功", body = Vec<crate::admin::dto::invitations::InvitationView>),
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
    res.render(Json(list_invitations().await));
}

/// 创建后台邀请码。
///
/// 仅 `owner` 可调用。邀请码用于“受邀页面”开户，不直接作为普通登录密码使用。
#[endpoint(
    tags("admin.invitations"),
    request_body = CreateInvitationRequest,
    responses(
        (status_code = 200, description = "创建邀请码成功", body = CreateInvitationResponse),
        (status_code = 403, description = "需要 owner 权限或尝试创建 owner 邀请码"),
        (status_code = 500, description = "创建邀请码失败")
    )
)]
async fn create(req: &mut Request, depot: &mut Depot, res: &mut Response) {
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

    let Ok(payload) = req.parse_json::<CreateInvitationRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };

    match create_invitation(
        current_admin.id,
        payload.role,
        payload.note,
        payload.expires_in_hours,
        client_ip(req),
        user_agent(req),
    )
    .await
    {
        Ok((invitation_token, invitation)) => res.render(Json(CreateInvitationResponse {
            invitation_token,
            invitation: crate::admin::dto::invitations::InvitationView {
                id: invitation.id,
                role: invitation.role,
                status: invitation.status,
                note: invitation.note,
                created_by_user_id: invitation.created_by_user_id,
                created_at: invitation.created_at,
                expires_at: invitation.expires_at,
                consumed_at: invitation.consumed_at,
                consumed_by_user_id: invitation.consumed_by_user_id,
            },
        })),
        Err("owner_invitation_forbidden") => render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_invitation_forbidden",
            "owner invitations are not allowed",
        ),
        Err(_) => render_api_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            "invitation_create_failed",
            "failed to create invitation",
        ),
    }
}

/// 撤销邀请码。
///
/// 仅 `owner` 可调用。只有 `pending` 状态的邀请码允许被撤销。
#[endpoint(
    tags("admin.invitations"),
    responses(
        (status_code = 200, description = "撤销邀请码成功"),
        (status_code = 400, description = "邀请码 ID 无效"),
        (status_code = 403, description = "需要 owner 权限"),
        (status_code = 404, description = "邀请码不存在"),
        (status_code = 409, description = "邀请码不是 pending 状态")
    )
)]
async fn revoke(req: &mut Request, depot: &mut Depot, res: &mut Response) {
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

    let Some(invitation_id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid invitation id",
        );
        return;
    };

    match revoke_invitation(
        invitation_id,
        current_admin.id,
        client_ip(req),
        user_agent(req),
    )
    .await
    {
        Ok(()) => res.render(Json(serde_json::json!({ "message": "revoked" }))),
        Err("invitation_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "invitation_not_found",
            "invitation does not exist",
        ),
        Err("invitation_not_pending") => render_api_error(
            res,
            StatusCode::CONFLICT,
            "invitation_not_pending",
            "invitation is not pending",
        ),
        Err(_) => render_api_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            "invitation_revoke_failed",
            "failed to revoke invitation",
        ),
    }
}

use salvo::prelude::*;
use sea_orm::Value;

use crate::admin::{
    db,
    entities::sessions,
    model::AdminRole,
    repository,
    services::{auth::client_ip, users::find_user_by_id},
};

pub const ADMIN_SESSION_COOKIE: &str = "irminsul_admin_session";
pub const DEPOT_ADMIN_USER: &str = "admin_user";
pub const DEPOT_ADMIN_SESSION: &str = "admin_session";

#[derive(Clone, Debug)]
pub struct CurrentAdmin {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub role: AdminRole,
    pub must_change_password: bool,
    pub must_change_username: bool,
    pub must_set_email: bool,
}

pub fn unix_timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

pub fn get_current_admin(depot: &Depot) -> Option<&CurrentAdmin> {
    depot.get(DEPOT_ADMIN_USER).ok()
}

pub fn require_role(depot: &Depot, required: AdminRole) -> bool {
    let Some(current_admin) = get_current_admin(depot) else {
        return false;
    };
    role_rank(&current_admin.role) <= role_rank(&required)
}

fn needs_profile_completion(current_admin: &CurrentAdmin) -> bool {
    current_admin.must_change_password
        || current_admin.must_change_username
        || current_admin.must_set_email
}

fn role_rank(role: &AdminRole) -> i32 {
    match role {
        AdminRole::Owner => 0,
        AdminRole::Admin => 1,
        AdminRole::Editor => 2,
        AdminRole::Viewer => 3,
    }
}

#[handler]
pub async fn require_authenticated_admin(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let Some(cookie) = req.cookie(ADMIN_SESSION_COOKIE) else {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "authentication required",
        );
        ctrl.skip_rest();
        return;
    };

    let token_hash = crate::admin::token::hash_token(cookie.value());
    let now = unix_timestamp();
    let Some(session) = find_session_by_token(db::database(), &token_hash, now).await else {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "session_invalid",
            "session is invalid or expired",
        );
        ctrl.skip_rest();
        return;
    };

    let Some(user) = find_user_by_id(db::database(), session.admin_user_id).await else {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "account_missing",
            "account no longer exists",
        );
        ctrl.skip_rest();
        return;
    };

    if user.status != crate::admin::model::AdminUserStatus::Active {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "account_disabled",
            "account is disabled",
        );
        ctrl.skip_rest();
        return;
    }

    depot.insert(
        DEPOT_ADMIN_USER,
        CurrentAdmin {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            must_change_password: user.must_change_password,
            must_change_username: user.must_change_username,
            must_set_email: user.must_set_email,
        },
    );
    depot.insert(DEPOT_ADMIN_SESSION, session);

    let _ = client_ip(req);
}

#[handler]
pub async fn require_completed_profile(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let Some(current_admin) = get_current_admin(depot) else {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "authentication required",
        );
        ctrl.skip_rest();
        return;
    };

    if !needs_profile_completion(current_admin) {
        return;
    }

    let path = req.uri().path();
    let allowed = path == "/api/admin/auth/me"
        || path == "/api/admin/auth/logout"
        || path == "/api/admin/auth/csrf"
        || path == "/api/admin/users/me";

    if !allowed {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "profile_completion_required",
            "profile completion is required before accessing this resource",
        );
        ctrl.skip_rest();
    }
}

pub async fn find_session_by_token(
    db: &sea_orm::DatabaseConnection,
    token_hash: &str,
    now: i64,
) -> Option<sessions::Model> {
    let row = repository::query_one(
        db,
        "SELECT id, admin_user_id, token_hash, csrf_token_hash, created_at, updated_at, expires_at, last_seen_at, revoked_at, created_ip, last_seen_ip, user_agent
         FROM ADMIN_SESSIONS
         WHERE token_hash = ?1 AND revoked_at IS NULL AND expires_at > ?2
         LIMIT 1",
        vec![Value::from(token_hash.to_string()), Value::from(now)],
    )
    .await
    .ok()??;
    repository::map_session_row(&row).ok()
}

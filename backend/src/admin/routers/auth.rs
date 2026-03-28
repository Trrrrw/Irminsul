use salvo::{
    Router,
    http::cookie::{Cookie, SameSite, time::Duration},
    prelude::*,
};

use crate::admin::{
    dto::auth::{AcceptInvitationRequest, AuthResponse, CsrfTokenResponse, LoginRequest},
    dto::users::AdminUserView,
    errors::render_api_error,
    middlewares::{
        auth::{
            ADMIN_SESSION_COOKIE, DEPOT_ADMIN_SESSION, get_current_admin,
            require_authenticated_admin,
        },
        csrf::require_csrf,
        origin::require_same_origin,
    },
    services::auth::{SESSION_TTL_SECONDS, accept_invitation, login, revoke_session, rotate_csrf},
};

pub fn router() -> Router {
    Router::with_path("api/admin/auth")
        .hoop(require_same_origin)
        .push(Router::with_path("login").post(login_handler))
        .push(Router::with_path("accept-invitation").post(accept_invitation_handler))
        .push(
            Router::new()
                .hoop(require_authenticated_admin)
                .push(Router::with_path("me").get(me))
                .push(Router::with_path("csrf").post(issue_csrf_token))
                .push(Router::with_path("logout").hoop(require_csrf).post(logout)),
        )
}

/// 管理员账号登录。
///
/// 使用用户名或邮箱加密码登录后台。登录成功后会设置 HttpOnly session cookie，
/// 并返回后续写请求必须携带的 CSRF token。
#[endpoint(
    tags("admin.auth"),
    request_body = LoginRequest,
    responses(
        (status_code = 200, description = "登录成功", body = AuthResponse),
        (status_code = 401, description = "用户名或密码错误"),
        (status_code = 403, description = "账号已禁用"),
        (status_code = 429, description = "登录失败次数过多")
    )
)]
async fn login_handler(req: &mut Request, res: &mut Response) {
    let Ok(payload) = req.parse_json::<LoginRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };

    match login(&payload.identifier, &payload.password, req).await {
        Ok((user, session_token, csrf_token)) => {
            set_session_cookie(req, res, &session_token);
            res.render(Json(AuthResponse {
                user: AdminUserView::from(user),
                csrf_token,
            }));
        }
        Err("too_many_attempts") => render_api_error(
            res,
            StatusCode::TOO_MANY_REQUESTS,
            "too_many_attempts",
            "too many failed login attempts",
        ),
        Err("account_disabled") => render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "account_disabled",
            "account is disabled",
        ),
        Err(_) => render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "invalid_credentials",
            "invalid credentials",
        ),
    }
}

/// 接受邀请码并创建后台账号。
///
/// 该接口用于“受邀页面”。用户只需提交邀请码，后端会创建一个带随机用户名和随机密码的账号，
/// 然后自动登录。首次登录后必须通过“修改自己的资料”接口完善用户名和密码。
#[endpoint(
    tags("admin.auth"),
    request_body = AcceptInvitationRequest,
    responses(
        (status_code = 200, description = "接受邀请码成功", body = AuthResponse),
        (status_code = 403, description = "邀请码已失效或已过期"),
        (status_code = 404, description = "邀请码不存在")
    )
)]
async fn accept_invitation_handler(req: &mut Request, res: &mut Response) {
    let Ok(payload) = req.parse_json::<AcceptInvitationRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };

    match accept_invitation(req, &payload.invitation_token).await {
        Ok((user, session_token, csrf_token)) => {
            set_session_cookie(req, res, &session_token);
            res.render(Json(AuthResponse {
                user: AdminUserView::from(user),
                csrf_token,
            }));
        }
        Err("invitation_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "invitation_not_found",
            "invitation does not exist",
        ),
        Err("invitation_expired") => render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "invitation_expired",
            "invitation has expired",
        ),
        Err(_) => render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "invitation_invalid",
            "invitation is no longer valid",
        ),
    }
}

/// 获取当前已登录管理员的信息。
///
/// 前端可在应用启动后调用此接口判断当前是否已登录，以及当前账号是否仍需补全资料。
#[endpoint(
    tags("admin.auth"),
    responses(
        (status_code = 200, description = "获取当前管理员成功", body = AdminUserView),
        (status_code = 401, description = "未登录")
    )
)]
async fn me(depot: &mut Depot, res: &mut Response) {
    let Some(current_admin) = get_current_admin(depot) else {
        render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "authentication required",
        );
        return;
    };

    res.render(Json(AdminUserView {
        id: current_admin.id,
        username: current_admin.username.clone(),
        email: current_admin.email.clone(),
        role: current_admin.role.clone(),
        status: crate::admin::model::AdminUserStatus::Active,
        must_change_password: current_admin.must_change_password,
        must_change_username: current_admin.must_change_username,
        must_set_email: current_admin.must_set_email,
    }));
}

/// 刷新当前会话的 CSRF token。
///
/// 所有后台写请求都必须携带 `X-CSRF-Token` 请求头。前端可在登录后或 token 失效时调用此接口。
#[endpoint(
    tags("admin.auth"),
    responses(
        (status_code = 200, description = "刷新 CSRF token 成功", body = CsrfTokenResponse),
        (status_code = 401, description = "未登录")
    )
)]
async fn issue_csrf_token(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let Ok(session) = depot
        .get::<crate::admin::entities::sessions::Model>(DEPOT_ADMIN_SESSION)
        .cloned()
    else {
        render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "authentication required",
        );
        return;
    };

    let csrf_token = rotate_csrf(&session, req).await;
    res.render(Json(CsrfTokenResponse { csrf_token }));
}

/// 注销当前后台会话。
///
/// 该接口会撤销当前 session，并清除浏览器中的 session cookie。
#[endpoint(
    tags("admin.auth"),
    responses(
        (status_code = 200, description = "注销成功"),
        (status_code = 401, description = "未登录"),
        (status_code = 403, description = "缺少或无效的 CSRF token")
    )
)]
async fn logout(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let Ok(session) = depot
        .get::<crate::admin::entities::sessions::Model>(DEPOT_ADMIN_SESSION)
        .cloned()
    else {
        render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "authentication required",
        );
        return;
    };

    revoke_session(
        &session,
        get_current_admin(depot).map(|value| value.id),
        req,
    )
    .await;
    clear_session_cookie(req, res);
    res.render(Json(serde_json::json!({
        "message": "logged out",
    })));
}

fn set_session_cookie(req: &Request, res: &mut Response, session_token: &str) {
    let mut cookie = Cookie::new(ADMIN_SESSION_COOKIE, session_token.to_string());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_path("/".to_string());
    cookie.set_max_age(Duration::seconds(SESSION_TTL_SECONDS));
    if request_is_secure(req) {
        cookie.set_secure(true);
    }
    let _ = res.add_cookie(cookie);
}

fn clear_session_cookie(req: &Request, res: &mut Response) {
    let mut cookie = Cookie::new(ADMIN_SESSION_COOKIE, String::new());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Strict);
    cookie.set_path("/".to_string());
    cookie.make_removal();
    if request_is_secure(req) {
        cookie.set_secure(true);
    }
    let _ = res.add_cookie(cookie);
}

fn request_is_secure(req: &Request) -> bool {
    req.header::<String>("x-forwarded-proto")
        .map(|proto| proto.eq_ignore_ascii_case("https"))
        .unwrap_or_else(|| {
            req.uri()
                .scheme_str()
                .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https"))
        })
}

use salvo::{http::Method, prelude::*};

use crate::admin::{entities::sessions, middlewares::auth::DEPOT_ADMIN_SESSION};

#[handler]
pub async fn require_csrf(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    if matches!(
        req.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS
    ) {
        return;
    }

    let Ok(session) = depot.get::<sessions::Model>(DEPOT_ADMIN_SESSION) else {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::UNAUTHORIZED,
            "auth_required",
            "authentication required",
        );
        ctrl.skip_rest();
        return;
    };

    let Some(csrf_token) = req.header::<String>("x-csrf-token") else {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "csrf_missing",
            "missing CSRF token",
        );
        ctrl.skip_rest();
        return;
    };

    let csrf_token_hash = crate::admin::token::hash_token(&csrf_token);
    if csrf_token_hash != session.csrf_token_hash {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "csrf_invalid",
            "invalid CSRF token",
        );
        ctrl.skip_rest();
    }
}

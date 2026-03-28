use salvo::{http::Method, prelude::*};

fn is_same_origin(req: &Request) -> bool {
    let Some(origin) = req.header::<String>("origin") else {
        return false;
    };

    let expected_origin = std::env::var("IRMINSUL_ADMIN_ORIGIN").unwrap_or_else(|_| {
        let scheme = req
            .header::<String>("x-forwarded-proto")
            .unwrap_or_else(|| req.uri().scheme_str().unwrap_or("http").to_string());
        let authority = req.header::<String>("x-forwarded-host").unwrap_or_else(|| {
            req.uri()
                .authority()
                .map(|value| value.to_string())
                .unwrap_or_default()
        });
        format!("{scheme}://{authority}")
    });

    origin == expected_origin
}

#[handler]
pub async fn require_same_origin(req: &mut Request, res: &mut Response, ctrl: &mut FlowCtrl) {
    if matches!(
        req.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS
    ) {
        return;
    }

    if !is_same_origin(req) {
        crate::admin::errors::render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "origin_check_failed",
            "origin check failed",
        );
        ctrl.skip_rest();
    }
}

use salvo::{http::StatusCode, prelude::*};
use serde::Serialize;

/// 统一的后台 API 错误响应结构。
#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: &'static str,
    pub message: String,
}

pub fn render_api_error(
    res: &mut Response,
    status_code: StatusCode,
    code: &'static str,
    message: impl Into<String>,
) {
    res.status_code(status_code);
    res.render(Json(ApiErrorBody {
        code,
        message: message.into(),
    }));
}

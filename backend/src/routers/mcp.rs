use salvo::{Router, handler};

pub fn router() -> Router {
    Router::with_path("mcp").post(mcp_handler)
}

#[handler]
pub async fn mcp_handler() -> &'static str {
    "MCP"
}

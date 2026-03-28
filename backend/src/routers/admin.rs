use salvo::{Router, prelude::StaticDir};

pub fn static_router() -> Router {
    Router::with_path("admin/{**path}").get(
        StaticDir::new("panel/dist")
            .defaults("index.html")
            .fallback("index.html")
            .auto_list(false),
    )
}

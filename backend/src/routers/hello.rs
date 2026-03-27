use salvo::{Router, oapi::endpoint};

pub fn router() -> Router {
    Router::with_path("hello").get(hello)
}

#[endpoint]
async fn hello() -> &'static str {
    "Hello, world!"
}

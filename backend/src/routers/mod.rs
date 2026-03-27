use rust_embed::RustEmbed;
use salvo::{
    Router,
    oapi::OpenApi,
    prelude::{Logger, Scalar},
    serve_static::{EmbeddedFileExt, static_embed},
};

mod admin;
mod hello;
mod mcp;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Assets;

pub fn root() -> Router {
    let favicon = Assets::get("favicon.ico")
        .expect("favicon not found")
        .into_handler();
    let router = Router::new()
        .hoop(Logger::new())
        .push(admin::router())
        .push(mcp::router())
        .push(hello::router())
        .push(Router::with_path("favicon.ico").get(favicon))
        .push(Router::with_path("assets/{**rest}").get(static_embed::<Assets>()));

    let doc = OpenApi::new("Irminsul", "0.0.1").merge_router(&router);
    router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(Scalar::new("/api-doc/openapi.json").into_router("scalar"))
}

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
    let public_router = Router::new()
        .hoop(Logger::new())
        .push(admin::static_router())
        .push(mcp::router())
        .push(hello::router())
        .push(Router::with_path("favicon.ico").get(favicon))
        .push(Router::with_path("assets/{**rest}").get(static_embed::<Assets>()));
    let admin_router = crate::admin::routers::router();

    let public_doc =
        OpenApi::new("Irminsul Public API", env!("CARGO_PKG_VERSION")).merge_router(&public_router);
    let mut router = public_router
        .push(public_doc.into_router("/api-doc/public/openapi.json"))
        .push(Scalar::new("/api-doc/public/openapi.json").into_router("docs/public"));

    if cfg!(debug_assertions) {
        let admin_doc = OpenApi::new("Irminsul Admin API", env!("CARGO_PKG_VERSION"))
            .merge_router(&admin_router);
        router = router
            .push(admin_router)
            .push(admin_doc.into_router("/api-doc/admin/openapi.json"))
            .push(Scalar::new("/api-doc/admin/openapi.json").into_router("docs/admin"));
    } else {
        router = router.push(admin_router);
    }

    router
}

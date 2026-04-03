use rust_embed::RustEmbed;
use salvo::{
    oapi::{OpenApi, Tag},
    prelude::{Logger, Scalar},
    serve_static::{static_embed, EmbeddedFileExt},
    Router,
};

mod admin;
mod hello;
mod mcp;

fn admin_doc_tags() -> [Tag; 3] {
    [
        Tag::new("admin.manage.schemas").description(
            "内容模型管理接口。\n\
            用来创建和维护 schema，也就是定义一类内容有哪些字段、是否启用 i18n、主记录与翻译记录如何关联。\n\
            当前端或运营在做“内容结构配置”时使用这一组接口。",
        ),
        Tag::new("admin.manage.entries").description(
            "前端内容管理主入口。\n\
            这是语言优先的业务接口，适合列表页、详情页、新建文档、补语言版本这些日常面板流程。\n\
            大多数内容录入与查看场景优先使用这一组接口。",
        ),
        Tag::new("admin.manage.collections").description(
            "底层原始文档接口。\n\
            这一组直接操作某个 schema 对应 collection 里的单条 document，不会自动处理 i18n 主记录与语言子记录关系。\n\
            适合后端调试、数据修复、脚本或非 i18n 内容场景；一般不建议作为前端多语言面板的主入口。",
        ),
    ]
}

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
        OpenApi::new("Irminsul API", env!("CARGO_PKG_VERSION")).merge_router(&public_router);
    let mut router = public_router
        .push(public_doc.into_router("/api-doc/public/openapi.json"))
        .push(Scalar::new("/api-doc/public/openapi.json").into_router("docs"));

    if cfg!(debug_assertions) {
        let admin_doc = OpenApi::new("Irminsul Admin API", env!("CARGO_PKG_VERSION"))
            .tags(admin_doc_tags())
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

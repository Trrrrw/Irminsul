use salvo::{Router, prelude::*};

use crate::{
    admin::{
        dto::content::{
            CreateDocumentLocaleRequest, CreateDocumentRequest, CreateLocalizedDocumentRequest,
            CreateLocalizedDocumentResponse, CreateSchemaRequest, DocumentView, EntryView,
            SchemaView, UpdateDocumentRequest, UpdateSchemaFieldsRequest, UpdateSchemaRequest,
        },
        errors::render_api_error,
        middlewares::{
            auth::{
                get_current_admin, require_authenticated_admin, require_completed_profile,
                require_role,
            },
            csrf::require_csrf,
            origin::require_same_origin,
        },
        model::AdminRole,
        services::audit::write_audit_log,
    },
    catalog::service::{self, ActorStamp},
};

pub fn router() -> Router {
    Router::with_path("api/admin/manage")
        .hoop(require_same_origin)
        .hoop(require_authenticated_admin)
        .hoop(require_completed_profile)
        .push(schemas_router())
        .push(collections_router())
        .push(entries_router())
}

fn schemas_router() -> Router {
    Router::with_path("schemas")
        .push(
            Router::new()
                .get(list_schemas)
                .push(Router::new().hoop(require_csrf).post(create_schema)),
        )
        .push(
            Router::with_path("{id}")
                .get(get_schema)
                .push(Router::new().hoop(require_csrf).patch(update_schema)),
        )
        .push(
            Router::with_path("{id}/fields")
                .hoop(require_csrf)
                .patch(update_schema_fields),
        )
}

fn collections_router() -> Router {
    Router::with_path("collections")
        .push(
            Router::with_path("{schema_key}").get(list_documents).push(
                Router::new()
                    .hoop(require_csrf)
                    .post(create_document)
                    .push(Router::with_path("localized").post(create_localized_document)),
            ),
        )
        .push(
            Router::with_path("{schema_key}/{document_id}")
                .get(get_document)
                .push(
                    Router::new()
                        .hoop(require_csrf)
                        .patch(update_document)
                        .delete(delete_document),
                ),
        )
}

fn entries_router() -> Router {
    Router::with_path("entries")
        .push(Router::with_path("{root_schema_key}").get(list_entries))
        .push(Router::with_path("{root_schema_key}/{root_document_id}").get(get_entry_detail))
        .push(
            Router::with_path("{schema_key}/localized")
                .hoop(require_csrf)
                .post(create_localized_document),
        )
        .push(
            Router::with_path("{translation_schema_key}/{root_document_id}/localized")
                .hoop(require_csrf)
                .post(create_document_locale),
        )
}

/// 获取 Schema 列表
///
/// 返回当前内容模型下已定义的全部 schema，按最近更新时间倒序排列
#[endpoint(tags("admin.manage.schemas"), responses((status_code = 200, body = Vec<SchemaView>)))]
async fn list_schemas(res: &mut Response) {
    match service::list_schemas().await {
        Ok(values) => res.render(Json(values)),
        Err(error) => render_api_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            "schema_list_failed",
            error,
        ),
    }
}

/// 创建 Schema
///
/// 创建一个新的动态内容 schema，并定义集合展示名、描述和字段结构
#[endpoint(
    tags("admin.manage.schemas"),
    request_body = CreateSchemaRequest,
    responses(
        (status_code = 200, description = "创建 schema 成功", body = SchemaView),
        (status_code = 400, description = "请求参数不合法或 schema key 冲突")
    )
)]
async fn create_schema(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Ok(payload) = req.parse_json::<CreateSchemaRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match service::create_schema(payload).await {
        Ok(schema) => {
            audit_change(
                req,
                depot,
                "create_schema",
                "schema",
                Some(schema.id.clone()),
                "created schema",
            )
            .await;
            res.render(Json(schema));
        }
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, "schema_create_failed", error),
    }
}

/// 获取 Schema 详情
///
/// 根据 schema 的 Mongo ObjectId 获取其完整定义
#[endpoint(
    tags("admin.manage.schemas"),
    parameters(
        ("id" = String, Path, description = "Schema 的 Mongo ObjectId")
    ),
    responses(
        (status_code = 200, description = "获取 schema 成功", body = SchemaView),
        (status_code = 404, description = "schema 不存在")
    )
)]
async fn get_schema(req: &mut Request, res: &mut Response) {
    let Some(id) = req.param::<String>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid schema id",
        );
        return;
    };
    match service::get_schema(&id).await {
        Ok(schema) => res.render(Json(schema)),
        Err(error) if error == "schema_not_found" => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "schema_not_found",
            "schema not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, "schema_get_failed", error),
    }
}

/// 更新 Schema 基本信息
///
/// 更新 schema 的显示名称和描述，不调整字段定义
#[endpoint(
    tags("admin.manage.schemas"),
    parameters(
        ("id" = String, Path, description = "Schema 的 Mongo ObjectId")
    ),
    request_body = UpdateSchemaRequest,
    responses(
        (status_code = 200, description = "更新 schema 成功", body = SchemaView),
        (status_code = 400, description = "请求参数不合法"),
        (status_code = 404, description = "schema 不存在")
    )
)]
async fn update_schema(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<String>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid schema id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpdateSchemaRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match service::update_schema(&id, payload).await {
        Ok(schema) => {
            audit_change(
                req,
                depot,
                "update_schema",
                "schema",
                Some(schema.id.clone()),
                "updated schema",
            )
            .await;
            res.render(Json(schema));
        }
        Err(error) if error == "schema_not_found" => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "schema_not_found",
            "schema not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, "schema_update_failed", error),
    }
}

/// 更新 Schema 字段定义
///
/// 整体替换 schema 的字段配置，用于调整动态集合的字段结构和排序
#[endpoint(
    tags("admin.manage.schemas"),
    parameters(
        ("id" = String, Path, description = "Schema 的 Mongo ObjectId")
    ),
    request_body = UpdateSchemaFieldsRequest,
    responses(
        (status_code = 200, description = "更新 schema 字段成功", body = SchemaView),
        (status_code = 400, description = "字段定义不合法"),
        (status_code = 404, description = "schema 不存在")
    )
)]
async fn update_schema_fields(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<String>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid schema id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpdateSchemaFieldsRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match service::update_schema_fields(&id, payload).await {
        Ok(schema) => {
            audit_change(
                req,
                depot,
                "update_schema_fields",
                "schema",
                Some(schema.id.clone()),
                "updated schema fields",
            )
            .await;
            res.render(Json(schema));
        }
        Err(error) if error == "schema_not_found" => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "schema_not_found",
            "schema not found",
        ),
        Err(error) => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "schema_fields_update_failed",
            error,
        ),
    }
}

/// 获取文档列表
///
/// 根据 schema key 查询某个动态集合下的文档，并支持父级、状态、启用状态和关键字过滤
#[endpoint(
    tags("admin.manage.collections"),
    parameters(
        ("schema_key" = String, Path, description = "目标集合对应的 schema key"),
        ("parent_id" = Option<String>, Query, description = "按父文档 ID 过滤"),
        ("keyword" = Option<String>, Query, description = "按 searchable 字段做关键字模糊过滤"),
        ("enabled" = Option<bool>, Query, description = "按启用状态过滤"),
        ("status" = Option<String>, Query, description = "按业务状态过滤"),
        ("page" = Option<u64>, Query, description = "页码，从 1 开始"),
        ("page_size" = Option<u64>, Query, description = "每页数量，范围 1-100")
    ),
    responses(
        (status_code = 200, description = "获取文档列表成功", body = Vec<DocumentView>),
        (status_code = 404, description = "schema 不存在")
    )
)]
async fn list_documents(req: &mut Request, res: &mut Response) {
    let Some(schema_key) = extract_collection_schema_key(req) else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_schema_key",
            "invalid schema key",
        );
        return;
    };
    let parent_id = req.query::<String>("parent_id");
    let keyword = req.query::<String>("keyword");
    let enabled = req.query::<bool>("enabled");
    let status = req.query::<String>("status");
    let page = req.query::<u64>("page").unwrap_or(1).max(1);
    let page_size = req.query::<u64>("page_size").unwrap_or(20).clamp(1, 100);

    match service::list_documents(&schema_key, None, None, None, None, None, None).await {
        Ok(values) => {
            let keyword = keyword.map(|value| value.to_ascii_lowercase());
            let mut values = values
                .into_iter()
                .filter(|document| {
                    parent_id
                        .as_ref()
                        .is_none_or(|value| document.parent_id.as_deref() == Some(value.as_str()))
                })
                .filter(|document| enabled.is_none_or(|value| document.enabled == value))
                .filter(|document| {
                    status
                        .as_ref()
                        .is_none_or(|value| document.status.eq_ignore_ascii_case(value))
                })
                .filter(|document| {
                    keyword.as_ref().is_none_or(|value| {
                        document
                            .fields
                            .to_string()
                            .to_ascii_lowercase()
                            .contains(value)
                    })
                })
                .collect::<Vec<_>>();

            let start = ((page - 1) * page_size) as usize;
            let end = start.saturating_add(page_size as usize).min(values.len());
            let page_values = if start >= values.len() {
                Vec::new()
            } else {
                values.drain(start..end).collect::<Vec<_>>()
            };
            res.render(Json(page_values));
        }
        Err(error) if error == "schema_not_found" => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "schema_not_found",
            "schema not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, "document_list_failed", error),
    }
}

/// 创建文档
///
/// 在指定 schema 对应的动态集合中新增一条业务文档
#[endpoint(
    tags("admin.manage.collections"),
    parameters(
        ("schema_key" = String, Path, description = "目标集合对应的 schema key")
    ),
    request_body = CreateDocumentRequest,
    responses(
        (status_code = 200, description = "创建文档成功", body = DocumentView),
        (status_code = 400, description = "请求参数不合法或字段校验失败"),
        (status_code = 404, description = "schema 不存在")
    )
)]
async fn create_document(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(schema_key) = req.param::<String>("schema_key") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_schema_key",
            "invalid schema key",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<CreateDocumentRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match service::create_document(&schema_key, payload, actor_stamp(depot)).await {
        Ok(document) => {
            audit_change(
                req,
                depot,
                "create_document",
                &schema_key,
                Some(document.id.clone()),
                "created document",
            )
            .await;
            res.render(Json(document));
        }
        Err(error) if error == "schema_not_found" => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "schema_not_found",
            "schema not found",
        ),
        Err(error) => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "document_create_failed",
            error,
        ),
    }
}

/// 首次创建文档
///
/// 这是语言优先内容流的起点。
/// 当前端进入“新建文档”页面时，用户先选择语言，再填写字段并提交到这里。
/// 后端会自动完成两步：
/// 1. 创建主记录
/// 2. 在该主记录下创建当前语言的首条子记录
///
/// 前端使用约定：
/// - 路径里的 `schema_key` 必须是翻译 schema 的 key，不是主 schema 的 key
/// - `root_fields` 放跨语言共享字段
/// - `fields` 放当前语言字段
#[endpoint(
    tags("admin.manage.entries"),
    parameters(
        ("schema_key" = String, Path, description = "翻译 schema 对应的 schema key")
    ),
    request_body = CreateLocalizedDocumentRequest,
    responses(
        (status_code = 200, description = "首次创建文档成功，会同时返回主记录和当前语言子记录", body = CreateLocalizedDocumentResponse),
        (status_code = 400, description = "schema 未配置 i18n，或请求参数不合法"),
        (status_code = 404, description = "schema 不存在")
    )
)]
async fn create_localized_document(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(schema_key) = req.param::<String>("schema_key") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_schema_key",
            "invalid schema key",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<CreateLocalizedDocumentRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };

    match service::create_localized_document(&schema_key, payload, actor_stamp(depot)).await {
        Ok(result) => {
            if let Some(actor) = get_current_admin(depot) {
                write_audit_log(
                    Some(actor.id),
                    Some(actor.username.clone()),
                    "create_localized_document",
                    &schema_key,
                    Some(result.localized_document.id.clone()),
                    "created root document and first localized document",
                    Some(serde_json::json!({
                        "root_document_id": result.root_document.id,
                        "localized_document_id": result.localized_document.id,
                        "parent_id": result.localized_document.parent_id,
                    })),
                    crate::admin::services::auth::client_ip(req),
                    crate::admin::services::auth::user_agent(req),
                )
                .await;
            }
            res.render(Json(result));
        }
        Err(error) if error == "schema_not_found" => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "schema_not_found",
            "schema not found",
        ),
        Err(error) => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "localized_document_create_failed",
            error,
        ),
    }
}

/// 获取文档列表（语言优先）
///
/// 这是面向前端列表页的语言优先接口。
/// 前端传入主 schema 和当前界面语言后，后端会返回：
/// - 主记录
/// - 当前语言对应的翻译记录（如果存在）
/// - 当前文档已有的语言列表
///
/// 前端推荐用法：
/// - 列表页加载时，总是调用这个接口，而不是直接查 translation schema
/// - 列表标题、摘要等文案优先取 `localized_document.fields`
/// - 如果 `localized_document = null`，说明当前语言还没建，可以在列表中提示“缺少该语言版本”
#[endpoint(
    tags("admin.manage.entries"),
    parameters(
        ("root_schema_key" = String, Path, description = "主记录 schema key，例如 games"),
        ("locale" = Option<String>, Query, description = "当前管理面板语言，默认 zh_cn"),
        ("enabled" = Option<bool>, Query, description = "按主记录启用状态过滤"),
        ("status" = Option<String>, Query, description = "按主记录状态过滤"),
        ("keyword" = Option<String>, Query, description = "对主记录与当前语言内容做关键字搜索"),
        ("page" = Option<u64>, Query, description = "页码，从 1 开始"),
        ("page_size" = Option<u64>, Query, description = "每页数量，范围 1-100")
    ),
    responses(
        (status_code = 200, description = "获取语言优先文档列表成功", body = Vec<EntryView>),
        (status_code = 404, description = "主 schema 或翻译 schema 不存在")
    )
)]
async fn list_entries(req: &mut Request, res: &mut Response) {
    let Some(root_schema_key) = extract_entries_root_schema_key(req) else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_schema_key",
            "invalid root schema key",
        );
        return;
    };

    let locale = req
        .query::<String>("locale")
        .unwrap_or_else(|| "zh_cn".to_string());
    match service::list_entries(
        &root_schema_key,
        &locale,
        req.query::<bool>("enabled"),
        req.query::<String>("status"),
        req.query::<String>("keyword"),
        req.query::<u64>("page"),
        req.query::<u64>("page_size"),
    )
    .await
    {
        Ok(entries) => res.render(Json(entries)),
        Err(error) if error == "schema_not_found" || error == "translation_schema_not_found" => {
            render_api_error(
                res,
                StatusCode::NOT_FOUND,
                "schema_not_found",
                "schema not found",
            )
        }
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, "entry_list_failed", error),
    }
}

/// 获取文档详情（语言优先）
///
/// 这是面向前端详情页的语言优先接口。
/// 前端只需要传入主记录 ID 和当前界面语言，后端会返回：
/// - 主记录
/// - 当前语言版本的详情（如果存在）
/// - 当前文档已有的全部语言列表
///
/// 前端推荐用法：
/// - 详情页默认带上当前管理面板语言请求这里
/// - 用户切换详情语言时，重新请求同一个主记录 ID，并改 `locale`
/// - 如果 `localized_document = null`，应展示“创建该语言版本”的入口
#[endpoint(
    tags("admin.manage.entries"),
    parameters(
        ("root_schema_key" = String, Path, description = "主记录 schema key，例如 games"),
        ("root_document_id" = String, Path, description = "主记录文档 ID"),
        ("locale" = Option<String>, Query, description = "当前要查看的语言，默认 zh_cn")
    ),
    responses(
        (status_code = 200, description = "获取语言优先文档详情成功", body = EntryView),
        (status_code = 404, description = "主 schema、翻译 schema或主记录不存在")
    )
)]
async fn get_entry_detail(req: &mut Request, res: &mut Response) {
    let Some((root_schema_key, root_document_id)) = extract_entry_detail_path(req) else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid root document id",
        );
        return;
    };

    let locale = req
        .query::<String>("locale")
        .unwrap_or_else(|| "zh_cn".to_string());
    match service::get_entry_detail(&root_schema_key, &root_document_id, &locale).await {
        Ok(entry) => res.render(Json(entry)),
        Err(error)
            if error == "schema_not_found"
                || error == "translation_schema_not_found"
                || error == "document_not_found" =>
        {
            render_api_error(
                res,
                StatusCode::NOT_FOUND,
                "entry_not_found",
                "entry not found",
            )
        }
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, "entry_get_failed", error),
    }
}

/// 为已有文档新增语言版本
///
/// 当详情页切换到某个还不存在的语言时，前端应调用这个接口。
/// 它只会在现有主记录下新增一个语言版本，不会重复创建主记录
///
/// 前端使用约定：
/// - 路径里的 `translation_schema_key` 必须是翻译 schema key
/// - 路径里的 `root_document_id` 必须是主记录 ID
/// - 提交成功后，前端可以直接再调用“获取文档详情（语言优先）”刷新当前语言内容
#[endpoint(
    tags("admin.manage.entries"),
    parameters(
        ("translation_schema_key" = String, Path, description = "翻译 schema key，例如 game_texts"),
        ("root_document_id" = String, Path, description = "主记录文档 ID")
    ),
    request_body = CreateDocumentLocaleRequest,
    responses(
        (status_code = 200, description = "新增语言版本成功", body = DocumentView),
        (status_code = 400, description = "语言已存在，或请求参数不合法"),
        (status_code = 404, description = "翻译 schema 或主记录不存在")
    )
)]
async fn create_document_locale(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(translation_schema_key) = req.param::<String>("translation_schema_key") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_schema_key",
            "invalid translation schema key",
        );
        return;
    };
    let Some(root_document_id) = req.param::<String>("root_document_id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid root document id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<CreateDocumentLocaleRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };

    match service::create_document_locale(
        &translation_schema_key,
        &root_document_id,
        payload,
        actor_stamp(depot),
    )
    .await
    {
        Ok(document) => {
            audit_change(
                req,
                depot,
                "create_document_locale",
                &translation_schema_key,
                Some(document.id.clone()),
                "created localized document under existing root document",
            )
            .await;
            res.render(Json(document));
        }
        Err(error)
            if error == "schema_not_found"
                || error == "translation_schema_not_found"
                || error == "document_not_found" =>
        {
            render_api_error(
                res,
                StatusCode::NOT_FOUND,
                "entry_not_found",
                "entry not found",
            )
        }
        Err(error) => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "document_locale_create_failed",
            error,
        ),
    }
}

/// 获取文档详情
///
/// 根据 schema key 和文档 ObjectId 获取单条业务文档
#[endpoint(
    tags("admin.manage.collections"),
    parameters(
        ("schema_key" = String, Path, description = "目标集合对应的 schema key"),
        ("document_id" = String, Path, description = "文档的 Mongo ObjectId")
    ),
    responses(
        (status_code = 200, description = "获取文档成功", body = DocumentView),
        (status_code = 404, description = "schema 或文档不存在")
    )
)]
async fn get_document(req: &mut Request, res: &mut Response) {
    let Some(schema_key) = req.param::<String>("schema_key") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_schema_key",
            "invalid schema key",
        );
        return;
    };
    let Some(document_id) = req.param::<String>("document_id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid document id",
        );
        return;
    };
    match service::get_document(&schema_key, &document_id).await {
        Ok(document) => res.render(Json(document)),
        Err(error) if error == "schema_not_found" || error == "document_not_found" => {
            render_api_error(
                res,
                StatusCode::NOT_FOUND,
                "document_not_found",
                "document not found",
            )
        }
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, "document_get_failed", error),
    }
}

/// 更新文档
///
/// 更新指定动态集合中的单条业务文档，未提交的字段保持原值
#[endpoint(
    tags("admin.manage.collections"),
    parameters(
        ("schema_key" = String, Path, description = "目标集合对应的 schema key"),
        ("document_id" = String, Path, description = "文档的 Mongo ObjectId")
    ),
    request_body = UpdateDocumentRequest,
    responses(
        (status_code = 200, description = "更新文档成功", body = DocumentView),
        (status_code = 400, description = "请求参数不合法或字段校验失败"),
        (status_code = 404, description = "schema 或文档不存在")
    )
)]
async fn update_document(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(schema_key) = req.param::<String>("schema_key") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_schema_key",
            "invalid schema key",
        );
        return;
    };
    let Some(document_id) = req.param::<String>("document_id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid document id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpdateDocumentRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match service::update_document(&schema_key, &document_id, payload, actor_stamp(depot)).await {
        Ok(document) => {
            audit_change(
                req,
                depot,
                "update_document",
                &schema_key,
                Some(document.id.clone()),
                "updated document",
            )
            .await;
            res.render(Json(document));
        }
        Err(error) if error == "schema_not_found" || error == "document_not_found" => {
            render_api_error(
                res,
                StatusCode::NOT_FOUND,
                "document_not_found",
                "document not found",
            )
        }
        Err(error) => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "document_update_failed",
            error,
        ),
    }
}

/// 删除文档
///
/// 根据 schema key 和文档 ObjectId 删除单条业务文档
#[endpoint(
    tags("admin.manage.collections"),
    parameters(
        ("schema_key" = String, Path, description = "目标集合对应的 schema key"),
        ("document_id" = String, Path, description = "文档的 Mongo ObjectId")
    ),
    responses(
        (status_code = 200, description = "删除文档成功"),
        (status_code = 404, description = "schema 或文档不存在")
    )
)]
async fn delete_document(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(schema_key) = req.param::<String>("schema_key") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_schema_key",
            "invalid schema key",
        );
        return;
    };
    let Some(document_id) = req.param::<String>("document_id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid document id",
        );
        return;
    };
    match service::delete_document(&schema_key, &document_id).await {
        Ok(()) => {
            audit_change(
                req,
                depot,
                "delete_document",
                &schema_key,
                Some(document_id),
                "deleted document",
            )
            .await;
            res.render(Json(serde_json::json!({ "message": "deleted" })));
        }
        Err(error) if error == "schema_not_found" || error == "document_not_found" => {
            render_api_error(
                res,
                StatusCode::NOT_FOUND,
                "document_not_found",
                "document not found",
            )
        }
        Err(error) => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "document_delete_failed",
            error,
        ),
    }
}

fn actor_stamp(depot: &Depot) -> Option<ActorStamp> {
    get_current_admin(depot).map(|actor| ActorStamp {
        user_id: actor.id,
        username: actor.username.clone(),
    })
}

async fn audit_change(
    req: &Request,
    depot: &Depot,
    action: &str,
    target_type: &str,
    target_id: Option<String>,
    summary: &str,
) {
    if let Some(actor) = get_current_admin(depot) {
        write_audit_log(
            Some(actor.id),
            Some(actor.username.clone()),
            action,
            target_type,
            target_id,
            summary,
            None,
            crate::admin::services::auth::client_ip(req),
            crate::admin::services::auth::user_agent(req),
        )
        .await;
    }
}

fn extract_collection_schema_key(req: &Request) -> Option<String> {
    const PREFIX: &str = "/api/admin/manage/collections/";

    let path = req.uri().path();
    let tail = path.strip_prefix(PREFIX)?;
    let schema_key = tail.split('/').next()?.trim();

    (!schema_key.is_empty()).then_some(schema_key.to_string())
}

fn extract_entries_root_schema_key(req: &Request) -> Option<String> {
    const PREFIX: &str = "/api/admin/manage/entries/";

    let path = req.uri().path();
    let tail = path.strip_prefix(PREFIX)?;
    let root_schema_key = tail.split('/').next()?.trim();

    (!root_schema_key.is_empty()).then_some(root_schema_key.to_string())
}

fn extract_entry_detail_path(req: &Request) -> Option<(String, String)> {
    const PREFIX: &str = "/api/admin/manage/entries/";

    let path = req.uri().path();
    let tail = path.strip_prefix(PREFIX)?;
    let mut segments = tail.split('/');
    let root_schema_key = segments.next()?.trim();
    let root_document_id = segments.next()?.trim();

    if root_schema_key.is_empty() || root_document_id.is_empty() {
        return None;
    }

    Some((root_schema_key.to_string(), root_document_id.to_string()))
}

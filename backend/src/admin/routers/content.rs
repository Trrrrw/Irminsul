use salvo::{Router, prelude::*};

use crate::{
    admin::{
        dto::content::{
            ContentItemDetailView, ContentItemView, ContentTypeMetadataView, ContentTypeView,
            CreateContentItemRequest, CreateContentTypeRequest, CreateGameRequest, GameDetailView,
            GameTextView, GameView, SetEnabledRequest, UpdateContentItemRequest,
            UpdateContentTypeRequest, UpdateGameRequest, UpsertContentItemTextRequest,
            UpsertContentTypeTextRequest, UpsertGameTextRequest,
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
    content::services::catalog,
    vector::jobs,
};

pub fn router() -> Router {
    Router::with_path("api/admin/manage")
        .hoop(require_same_origin)
        .hoop(require_authenticated_admin)
        .hoop(require_completed_profile)
        .push(games_router())
        .push(content_types_router())
        .push(content_items_router())
}

fn games_router() -> Router {
    Router::with_path("games")
        .push(
            Router::new()
                .get(list_games)
                .push(Router::new().hoop(require_csrf).post(create_game)),
        )
        .push(
            Router::with_path("<id>")
                .get(get_game_detail)
                .hoop(require_csrf)
                .patch(update_game),
        )
        .push(
            Router::with_path("<id>/status")
                .hoop(require_csrf)
                .patch(set_game_status),
        )
        .push(
            Router::with_path("<id>/texts/<locale>")
                .hoop(require_csrf)
                .put(upsert_game_text)
                .delete(delete_game_text),
        )
}

fn content_types_router() -> Router {
    Router::with_path("content-types")
        .push(
            Router::new()
                .get(list_content_types)
                .push(Router::new().hoop(require_csrf).post(create_content_type)),
        )
        .push(
            Router::with_path("<id>")
                .get(get_content_type_metadata)
                .hoop(require_csrf)
                .patch(update_content_type),
        )
        .push(
            Router::with_path("<id>/status")
                .hoop(require_csrf)
                .patch(set_content_type_status),
        )
        .push(
            Router::with_path("<id>/texts/<locale>")
                .hoop(require_csrf)
                .put(upsert_content_type_text)
                .delete(delete_content_type_text),
        )
}

fn content_items_router() -> Router {
    Router::with_path("content-items")
        .push(
            Router::new()
                .get(list_content_items)
                .push(Router::new().hoop(require_csrf).post(create_content_item)),
        )
        .push(
            Router::with_path("<id>")
                .get(get_content_item_detail)
                .hoop(require_csrf)
                .patch(update_content_item)
                .delete(delete_content_item),
        )
        .push(
            Router::with_path("<id>/texts/<locale>")
                .hoop(require_csrf)
                .put(upsert_content_item_text)
                .delete(delete_content_item_text),
        )
        .push(
            Router::with_path("<id>/embedding/rebuild")
                .hoop(require_csrf)
                .post(rebuild_embeddings),
        )
}

/// 获取游戏列表
#[endpoint(
    tags("admin.manage.games"),
    responses((status_code = 200, description = "获取游戏列表成功", body = Vec<GameView>))
)]
async fn list_games(req: &mut Request, res: &mut Response) {
    let locale = req.query::<String>("locale");
    match catalog::list_games(locale.as_deref()).await {
        Ok(values) => res.render(Json(values)),
        Err(error) => render_api_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            "games_list_failed",
            error,
        ),
    }
}

/// 创建游戏
///
/// 该接口会同时创建游戏主记录，以及 `locale` 指定的第一条多语言文本
/// 如果后续还要补英文、日文等其他语言版本，请调用 `/api/admin/manage/games/{id}/texts/{locale}`
#[endpoint(
    tags("admin.manage.games"),
    request_body = CreateGameRequest,
    responses((status_code = 200, description = "创建游戏成功", body = GameView))
)]
async fn create_game(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Ok(payload) = req.parse_json::<CreateGameRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match catalog::create_game(payload).await {
        Ok(game) => {
            audit_content_change(req, depot, "create_game", "game", game.id, "created game").await;
            res.render(Json(game));
        }
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 更新游戏
#[endpoint(
    tags("admin.manage.games"),
    request_body = UpdateGameRequest,
    responses((status_code = 200, description = "更新游戏成功", body = GameView))
)]
async fn update_game(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid game id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpdateGameRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match catalog::update_game(id, payload).await {
        Ok(game) => {
            audit_content_change(req, depot, "update_game", "game", game.id, "updated game").await;
            res.render(Json(game));
        }
        Err("game_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "game_not_found",
            "game not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 获取游戏详情
#[endpoint(
    tags("admin.manage.games"),
    responses((status_code = 200, description = "获取游戏详情成功", body = GameDetailView))
)]
async fn get_game_detail(req: &mut Request, res: &mut Response) {
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid game id",
        );
        return;
    };
    let locale = req.query::<String>("locale");
    match catalog::get_game_detail(id, locale.as_deref()).await {
        Ok(detail) => res.render(Json(detail)),
        Err("game_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "game_not_found",
            "game not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 更新游戏启用状态
#[endpoint(
    tags("admin.manage.games"),
    request_body = SetEnabledRequest,
    responses((status_code = 200, description = "更新游戏状态成功", body = GameView))
)]
async fn set_game_status(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid game id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<SetEnabledRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match catalog::set_game_enabled(id, payload).await {
        Ok(game) => {
            audit_content_change(
                req,
                depot,
                "set_game_status",
                "game",
                game.id,
                "updated game status",
            )
            .await;
            res.render(Json(game));
        }
        Err("game_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "game_not_found",
            "game not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 新增或更新指定语言的游戏文本
///
/// 用于给已存在的游戏补充或修改某个语言版本，例如英文名、日文简介等
#[endpoint(
    tags("admin.manage.games"),
    request_body = UpsertGameTextRequest,
    responses((status_code = 200, description = "保存游戏文本成功", body = GameTextView))
)]
async fn upsert_game_text(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid game id",
        );
        return;
    };
    let Some(locale) = req.param::<String>("locale") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_locale",
            "invalid locale",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpsertGameTextRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    let Ok(locale) = locale.parse() else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "unsupported_locale",
            "unsupported locale",
        );
        return;
    };
    match catalog::upsert_game_text(id, locale, payload).await {
        Ok(text) => {
            audit_content_change(
                req,
                depot,
                "upsert_game_text",
                "game_text",
                text.id,
                "saved localized game text",
            )
            .await;
            res.render(Json(text));
        }
        Err("game_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "game_not_found",
            "game not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 删除指定语言的游戏文本
#[endpoint(
    tags("admin.manage.games"),
    responses((status_code = 200, description = "删除游戏文本成功"))
)]
async fn delete_game_text(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid game id",
        );
        return;
    };
    let Some(locale) = req.param::<String>("locale") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_locale",
            "invalid locale",
        );
        return;
    };
    match catalog::delete_game_text(id, &locale).await {
        Ok(()) => {
            audit_content_change(
                req,
                depot,
                "delete_game_text",
                "game_text",
                id,
                "deleted localized game text",
            )
            .await;
            res.render(Json(serde_json::json!({ "message": "deleted" })));
        }
        Err("game_text_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "game_text_not_found",
            "game text not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 获取内容类型列表
#[endpoint(
    tags("admin.manage.content-types"),
    responses((status_code = 200, description = "获取内容类型列表成功", body = Vec<ContentTypeView>))
)]
async fn list_content_types(req: &mut Request, res: &mut Response) {
    let game_id = req.query::<i64>("game_id");
    let locale = req.query::<String>("locale");
    match catalog::list_content_types(game_id, locale.as_deref()).await {
        Ok(values) => res.render(Json(values)),
        Err(error) => render_api_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            "content_types_list_failed",
            error,
        ),
    }
}

/// 创建内容类型
///
/// 该接口会同时创建内容类型主记录，以及 `locale` 指定的第一条多语言文本
/// 如果后续还要补其他语言版本，请调用 `/api/admin/manage/content-types/{id}/texts/{locale}`
#[endpoint(
    tags("admin.manage.content-types"),
    request_body = CreateContentTypeRequest,
    responses((status_code = 200, description = "创建内容类型成功", body = ContentTypeView))
)]
async fn create_content_type(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Ok(payload) = req.parse_json::<CreateContentTypeRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match catalog::create_content_type(payload).await {
        Ok(content_type) => {
            audit_content_change(
                req,
                depot,
                "create_content_type",
                "content_type",
                content_type.id,
                "created content type",
            )
            .await;
            res.render(Json(content_type));
        }
        Err("game_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "game_not_found",
            "game not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 获取内容类型元信息
#[endpoint(
    tags("admin.manage.content-types"),
    responses((status_code = 200, description = "获取内容类型元信息成功", body = ContentTypeMetadataView))
)]
async fn get_content_type_metadata(req: &mut Request, res: &mut Response) {
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content type id",
        );
        return;
    };
    let locale = req.query::<String>("locale");
    match catalog::get_content_type_metadata(id, locale.as_deref()).await {
        Ok(metadata) => res.render(Json(metadata)),
        Err("content_type_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_type_not_found",
            "content type not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 新增或更新指定语言的内容类型文本
///
/// 用于给已存在的内容类型补充或修改某个语言版本，例如中文名、英文说明等
#[endpoint(
    tags("admin.manage.content-types"),
    request_body = UpsertContentTypeTextRequest,
    responses((status_code = 200, description = "保存内容类型文本成功", body = crate::content::dto::content::ContentTypeTextView))
)]
async fn upsert_content_type_text(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content type id",
        );
        return;
    };
    let Some(locale) = req.param::<String>("locale") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_locale",
            "invalid locale",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpsertContentTypeTextRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    let Ok(locale) = locale.parse() else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "unsupported_locale",
            "unsupported locale",
        );
        return;
    };
    match catalog::upsert_content_type_text(id, locale, payload).await {
        Ok(text) => {
            audit_content_change(
                req,
                depot,
                "upsert_content_type_text",
                "content_type_text",
                text.id,
                "saved localized content type text",
            )
            .await;
            res.render(Json(text));
        }
        Err("content_type_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_type_not_found",
            "content type not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 删除指定语言的内容类型文本
#[endpoint(
    tags("admin.manage.content-types"),
    responses((status_code = 200, description = "删除内容类型文本成功"))
)]
async fn delete_content_type_text(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content type id",
        );
        return;
    };
    let Some(locale) = req.param::<String>("locale") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_locale",
            "invalid locale",
        );
        return;
    };
    match catalog::delete_content_type_text(id, &locale).await {
        Ok(()) => {
            audit_content_change(
                req,
                depot,
                "delete_content_type_text",
                "content_type_text",
                id,
                "deleted localized content type text",
            )
            .await;
            res.render(Json(serde_json::json!({ "message": "deleted" })));
        }
        Err("content_type_text_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_type_text_not_found",
            "content type text not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 更新内容类型
#[endpoint(
    tags("admin.manage.content-types"),
    request_body = UpdateContentTypeRequest,
    responses((status_code = 200, description = "更新内容类型成功", body = ContentTypeView))
)]
async fn update_content_type(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content type id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpdateContentTypeRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match catalog::update_content_type(id, payload).await {
        Ok(content_type) => {
            audit_content_change(
                req,
                depot,
                "update_content_type",
                "content_type",
                content_type.id,
                "updated content type",
            )
            .await;
            res.render(Json(content_type));
        }
        Err("content_type_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_type_not_found",
            "content type not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 更新内容类型启用状态
#[endpoint(
    tags("admin.manage.content-types"),
    request_body = SetEnabledRequest,
    responses((status_code = 200, description = "更新内容类型状态成功", body = ContentTypeView))
)]
async fn set_content_type_status(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Owner) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "owner_required",
            "owner role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content type id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<SetEnabledRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match catalog::set_content_type_enabled(id, payload).await {
        Ok(content_type) => {
            audit_content_change(
                req,
                depot,
                "set_content_type_status",
                "content_type",
                content_type.id,
                "updated content type status",
            )
            .await;
            res.render(Json(content_type));
        }
        Err("content_type_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_type_not_found",
            "content type not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 获取内容实例列表
#[endpoint(
    tags("admin.manage.content-items"),
    responses((status_code = 200, description = "获取内容实例列表成功", body = Vec<ContentItemView>))
)]
async fn list_content_items(req: &mut Request, res: &mut Response) {
    let game_id = req.query::<i64>("game_id");
    let content_type_id = req.query::<i64>("content_type_id");
    let locale = req.query::<String>("locale");
    match catalog::list_content_items(game_id, content_type_id, locale.as_deref()).await {
        Ok(values) => res.render(Json(values)),
        Err(error) => render_api_error(
            res,
            StatusCode::INTERNAL_SERVER_ERROR,
            "content_items_list_failed",
            error,
        ),
    }
}

/// 创建内容实例
///
/// 该接口只创建语言无关的内容实例主记录
/// 具体的名称、简介、正文等多语言内容需要通过 `/api/admin/manage/content-items/{id}/texts/{locale}` 维护
#[endpoint(
    tags("admin.manage.content-items"),
    request_body = CreateContentItemRequest,
    responses((status_code = 200, description = "创建内容实例成功", body = ContentItemView))
)]
async fn create_content_item(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Ok(payload) = req.parse_json::<CreateContentItemRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match catalog::create_content_item(payload).await {
        Ok(item) => {
            audit_content_change(
                req,
                depot,
                "create_content_item",
                "content_item",
                item.id,
                "created content item",
            )
            .await;
            res.render(Json(item));
        }
        Err("game_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "game_not_found",
            "game not found",
        ),
        Err("content_type_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_type_not_found",
            "content type not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 获取内容实例详情
#[endpoint(
    tags("admin.manage.content-items"),
    responses((status_code = 200, description = "获取内容实例详情成功", body = ContentItemDetailView))
)]
async fn get_content_item_detail(req: &mut Request, res: &mut Response) {
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content item id",
        );
        return;
    };
    let locale = req.query::<String>("locale");
    match catalog::get_content_item_detail(id, locale.as_deref()).await {
        Ok(detail) => res.render(Json(detail)),
        Err("content_item_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_item_not_found",
            "content item not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 更新内容实例
#[endpoint(
    tags("admin.manage.content-items"),
    request_body = UpdateContentItemRequest,
    responses((status_code = 200, description = "更新内容实例成功", body = ContentItemView))
)]
async fn update_content_item(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content item id",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpdateContentItemRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    match catalog::update_content_item(id, payload).await {
        Ok(item) => {
            audit_content_change(
                req,
                depot,
                "update_content_item",
                "content_item",
                item.id,
                "updated content item",
            )
            .await;
            res.render(Json(item));
        }
        Err("content_item_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_item_not_found",
            "content item not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 删除内容实例
#[endpoint(
    tags("admin.manage.content-items"),
    responses((status_code = 200, description = "删除内容实例成功"))
)]
async fn delete_content_item(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content item id",
        );
        return;
    };
    match catalog::delete_content_item(id).await {
        Ok(()) => {
            audit_content_change(
                req,
                depot,
                "delete_content_item",
                "content_item",
                id,
                "deleted content item",
            )
            .await;
            res.render(Json(serde_json::json!({ "message": "deleted" })));
        }
        Err("content_item_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_item_not_found",
            "content item not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 新增或更新指定语言的内容文本
///
/// 用于给已存在的内容实例补充或修改某个语言版本的文本内容
#[endpoint(
    tags("admin.manage.content-items"),
    request_body = UpsertContentItemTextRequest,
    responses((status_code = 200, description = "保存内容文本成功", body = crate::content::dto::content::ContentItemTextView))
)]
async fn upsert_content_item_text(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content item id",
        );
        return;
    };
    let Some(locale) = req.param::<String>("locale") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_locale",
            "invalid locale",
        );
        return;
    };
    let Ok(payload) = req.parse_json::<UpsertContentItemTextRequest>().await else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "invalid JSON payload",
        );
        return;
    };
    let actor = get_current_admin(depot).cloned();
    match catalog::upsert_content_item_text(
        id,
        &locale,
        payload,
        actor.as_ref().map(|value| value.id),
        actor.as_ref().map(|value| value.username.clone()),
    )
    .await
    {
        Ok(text) => {
            audit_content_change(
                req,
                depot,
                "upsert_content_item_text",
                "content_item_text",
                text.id,
                "saved localized content text",
            )
            .await;
            res.render(Json(text));
        }
        Err("content_item_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_item_not_found",
            "content item not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 删除指定语言的内容文本
#[endpoint(
    tags("admin.manage.content-items"),
    responses((status_code = 200, description = "删除内容文本成功"))
)]
async fn delete_content_item_text(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content item id",
        );
        return;
    };
    let Some(locale) = req.param::<String>("locale") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_locale",
            "invalid locale",
        );
        return;
    };
    match catalog::delete_content_item_text(id, &locale).await {
        Ok(()) => {
            audit_content_change(
                req,
                depot,
                "delete_content_item_text",
                "content_item_text",
                id,
                "deleted localized content text",
            )
            .await;
            res.render(Json(serde_json::json!({ "message": "deleted" })));
        }
        Err("content_item_text_not_found") => render_api_error(
            res,
            StatusCode::NOT_FOUND,
            "content_item_text_not_found",
            "content item text not found",
        ),
        Err(error) => render_api_error(res, StatusCode::BAD_REQUEST, error, error),
    }
}

/// 手动触发某个内容实例的向量重建
#[endpoint(
    tags("admin.manage.content-items"),
    responses((status_code = 200, description = "触发向量重建成功"))
)]
async fn rebuild_embeddings(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    if !require_role(depot, AdminRole::Editor) {
        render_api_error(
            res,
            StatusCode::FORBIDDEN,
            "editor_required",
            "editor role is required",
        );
        return;
    }
    let Some(id) = req.param::<i64>("id") else {
        render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "invalid_id",
            "invalid content item id",
        );
        return;
    };
    let actor = get_current_admin(depot).cloned();
    match jobs::enqueue_for_item(
        id,
        actor.as_ref().map(|value| value.id),
        actor.as_ref().map(|value| value.username.clone()),
    )
    .await
    {
        Ok(count) => {
            audit_content_change(
                req,
                depot,
                "rebuild_content_embeddings",
                "content_item",
                id,
                "manually triggered embedding rebuild",
            )
            .await;
            res.render(Json(serde_json::json!({ "enqueued": count })));
        }
        Err(error) => render_api_error(
            res,
            StatusCode::BAD_REQUEST,
            "embedding_rebuild_failed",
            error,
        ),
    }
}

async fn audit_content_change(
    req: &Request,
    depot: &Depot,
    action: &str,
    target_type: &str,
    target_id: i64,
    summary: &str,
) {
    if let Some(actor) = get_current_admin(depot) {
        write_audit_log(
            Some(actor.id),
            Some(actor.username.clone()),
            action,
            target_type,
            Some(target_id.to_string()),
            summary,
            None,
            crate::admin::services::auth::client_ip(req),
            crate::admin::services::auth::user_agent(req),
        )
        .await;
    }
}

use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

use crate::content::model::{ContentItemStatus, EmbeddingJobStatus, Locale};

/// 游戏视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct GameView {
    pub id: i64,
    pub code: String,
    pub display_name: Option<String>,
    pub display_description: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 游戏多语言文本视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct GameTextView {
    pub id: i64,
    pub game_id: i64,
    pub locale: Locale,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 游戏详情视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct GameDetailView {
    pub game: GameView,
    pub localized_texts: Vec<GameTextView>,
    pub missing_locales: Vec<String>,
}

/// 创建游戏请求。
///
/// `locale`、`name`、`description` 表示创建时附带写入的第一种语言文本。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateGameRequest {
    pub code: String,
    pub locale: Locale,
    pub name: String,
    pub description: Option<String>,
}

/// 更新游戏请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpdateGameRequest {
    pub code: Option<String>,
}

/// 新增或更新游戏多语言文本请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpsertGameTextRequest {
    pub name: String,
    pub description: Option<String>,
}

/// 更新启用状态请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct SetEnabledRequest {
    pub enabled: bool,
}

/// 内容类型视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ContentTypeView {
    pub id: i64,
    pub game_id: i64,
    pub code: String,
    pub display_name: Option<String>,
    pub display_description: Option<String>,
    pub supports_i18n: bool,
    pub supports_embedding: bool,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 内容类型多语言文本视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ContentTypeTextView {
    pub id: i64,
    pub content_type_id: i64,
    pub locale: Locale,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 内容类型元信息。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ContentTypeMetadataView {
    pub content_type: ContentTypeView,
    pub localized_texts: Vec<ContentTypeTextView>,
    pub missing_locales: Vec<String>,
    pub supported_locales: Vec<String>,
    pub core_fields: Vec<String>,
}

/// 创建内容类型请求。
///
/// `locale`、`name`、`description` 表示创建时附带写入的第一种语言文本。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateContentTypeRequest {
    pub game_id: i64,
    pub code: String,
    pub locale: Locale,
    pub name: String,
    pub description: Option<String>,
    pub supports_i18n: bool,
    pub supports_embedding: bool,
}

/// 更新内容类型请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpdateContentTypeRequest {
    pub code: Option<String>,
    pub supports_i18n: Option<bool>,
    pub supports_embedding: Option<bool>,
}

/// 新增或更新内容类型多语言文本请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpsertContentTypeTextRequest {
    pub name: String,
    pub description: Option<String>,
}

/// 通用内容实例视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ContentItemView {
    pub id: i64,
    pub game_id: i64,
    pub content_type_id: i64,
    pub display_name: Option<String>,
    pub slug: Option<String>,
    pub external_key: Option<String>,
    pub status: ContentItemStatus,
    pub sort_order: i64,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 通用内容实例详情。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ContentItemDetailView {
    pub item: ContentItemView,
    pub game: GameView,
    pub content_type: ContentTypeView,
    pub localized_texts: Vec<ContentItemTextView>,
    pub missing_locales: Vec<String>,
    pub latest_embedding_job: Option<EmbeddingJobView>,
}

/// 创建内容实例请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateContentItemRequest {
    pub game_id: i64,
    pub content_type_id: i64,
    pub slug: Option<String>,
    pub external_key: Option<String>,
    pub status: ContentItemStatus,
    pub sort_order: Option<i64>,
}

/// 更新内容实例请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpdateContentItemRequest {
    pub slug: Option<String>,
    pub external_key: Option<String>,
    pub status: Option<ContentItemStatus>,
    pub sort_order: Option<i64>,
}

/// 多语言文本视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct ContentItemTextView {
    pub id: i64,
    pub content_item_id: i64,
    pub locale: Locale,
    pub name: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub body: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 新增或更新某一语言文本请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpsertContentItemTextRequest {
    pub name: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub body: Option<String>,
}

/// 向量任务视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct EmbeddingJobView {
    pub id: i64,
    pub content_item_id: i64,
    pub content_item_text_id: i64,
    pub trigger_reason: String,
    pub status: EmbeddingJobStatus,
    pub model: String,
    pub error_message: Option<String>,
    pub attempt_count: i64,
    pub requested_by_user_id: Option<i64>,
    pub requested_by_label: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
}

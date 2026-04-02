use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

/// Embedding 提供方视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct EmbeddingProviderView {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub base_url: String,
    pub embeddings_path: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 设置页中的 embedding 提供方写入项。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct EmbeddingProviderInput {
    pub id: Option<i64>,
    pub code: String,
    pub name: String,
    pub base_url: String,
    pub embeddings_path: Option<String>,
    pub enabled: bool,
}

/// Embedding API key 视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct EmbeddingApiKeyView {
    pub id: i64,
    pub provider_id: i64,
    pub name: String,
    pub masked_api_key: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 设置页中的 embedding API key 写入项。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct EmbeddingApiKeyInput {
    pub id: Option<i64>,
    pub provider_id: i64,
    pub name: String,
    pub api_key: Option<String>,
    pub enabled: bool,
}

/// Embedding 设置视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct EmbeddingSettingsView {
    pub default_provider_id: Option<i64>,
    pub current_model: String,
    pub updated_at: i64,
}

/// Embedding 设置页聚合视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct EmbeddingSettingsSectionView {
    pub settings: EmbeddingSettingsView,
    pub providers: Vec<EmbeddingProviderView>,
    pub api_keys: Vec<EmbeddingApiKeyView>,
}

/// 后台设置页聚合视图。
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct AdminSettingsView {
    pub embedding: EmbeddingSettingsSectionView,
}

/// 更新 embedding 设置请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpdateEmbeddingSettingsRequest {
    pub default_provider_id: Option<i64>,
    pub current_model: String,
}

/// 设置页中的 embedding 模块更新请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpdateEmbeddingSectionRequest {
    pub settings: Option<UpdateEmbeddingSettingsRequest>,
    pub providers: Option<Vec<EmbeddingProviderInput>>,
    pub api_keys: Option<Vec<EmbeddingApiKeyInput>>,
}

/// 后台设置页更新请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpdateAdminSettingsRequest {
    pub embedding: Option<UpdateEmbeddingSectionRequest>,
}

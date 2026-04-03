use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 动态字段类型。
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaFieldType {
    String,
    Integer,
    Float,
    Boolean,
    Object,
    Array,
}

/// Schema 的 i18n 模式。
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaI18nMode {
    Neutral,
    Root,
    Translation,
}

/// Schema 的 i18n 配置。
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct SchemaI18nConfig {
    /// i18n 模式。
    /// `neutral` 表示普通 schema；
    /// `root` 表示主记录 schema；
    /// `translation` 表示语言子记录 schema。
    pub mode: SchemaI18nMode,
    /// 当 `mode = translation` 时必填。
    /// 指向所属主记录 schema 的 key，例如 `games`。
    pub root_schema_key: Option<String>,
    /// 当 `mode = translation` 时必填。
    /// 指定语言字段在当前 schema 中的字段 key，例如 `locale`。
    pub locale_field: Option<String>,
}

/// 集合字段定义。
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct SchemaFieldDefinition {
    /// 字段 key，会作为 `fields` 内的属性名使用。
    pub key: String,
    /// 字段中文名称，主要用于管理面展示。
    pub label: String,
    /// 字段类型。
    pub field_type: SchemaFieldType,
    /// 是否必填。
    pub required: bool,
    /// 是否参与关键字搜索。
    pub searchable: bool,
    /// 是否允许排序。
    pub sortable: bool,
    /// 默认值。
    pub default_value: Option<Value>,
    /// 可选项列表，通常用于字符串枚举字段。
    pub options: Option<Vec<String>>,
    /// 引用信息，当前仅做配置透传。
    pub references: Option<String>,
    /// 排序号，越小越靠前。
    pub order: i32,
}

/// 集合元数据视图。
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct SchemaView {
    /// Schema 的 Mongo ObjectId。
    pub id: String,
    /// Schema 唯一 key，例如 `games`、`game_texts`。
    pub key: String,
    /// Schema 显示名称。
    pub display_name: String,
    /// Schema 描述。
    pub description: Option<String>,
    /// i18n 配置。
    pub i18n: Option<SchemaI18nConfig>,
    /// 字段定义列表。
    pub fields: Vec<SchemaFieldDefinition>,
    /// 创建时间，Unix 时间戳（秒）。
    pub created_at: i64,
    /// 更新时间，Unix 时间戳（秒）。
    pub updated_at: i64,
}

/// 创建集合请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateSchemaRequest {
    /// Schema 唯一 key。
    pub key: String,
    /// Schema 显示名称。
    pub display_name: String,
    /// Schema 描述。
    pub description: Option<String>,
    /// i18n 配置。
    pub i18n: Option<SchemaI18nConfig>,
    /// 字段定义列表。
    pub fields: Option<Vec<SchemaFieldDefinition>>,
}

/// 更新集合基础信息请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpdateSchemaRequest {
    /// 新的显示名称。
    pub display_name: Option<String>,
    /// 新的描述。
    pub description: Option<String>,
    /// 新的 i18n 配置。
    pub i18n: Option<SchemaI18nConfig>,
}

/// 更新集合字段定义请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpdateSchemaFieldsRequest {
    /// 完整字段定义列表，会整体替换原配置。
    pub fields: Vec<SchemaFieldDefinition>,
}

/// 动态文档视图。
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct DocumentView {
    /// 文档 Mongo ObjectId。
    pub id: String,
    /// 文档所属 schema key。
    pub schema_key: String,
    /// 父文档 ID。
    /// 对于语言子记录，这里是主记录 ID。
    pub parent_id: Option<String>,
    /// 业务状态，例如 `draft`、`published`。
    pub status: String,
    /// 是否启用。
    pub enabled: bool,
    /// 创建时间，Unix 时间戳（秒）。
    pub created_at: i64,
    /// 更新时间，Unix 时间戳（秒）。
    pub updated_at: i64,
    /// 创建人用户名。
    pub created_by: Option<String>,
    /// 最后更新人用户名。
    pub updated_by: Option<String>,
    /// 动态业务字段。
    pub fields: Value,
}

/// 创建文档请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateDocumentRequest {
    /// 父文档 ID。
    pub parent_id: Option<String>,
    /// 初始业务状态。
    pub status: Option<String>,
    /// 初始启用状态。
    pub enabled: Option<bool>,
    /// 文档字段内容，键必须与 schema.fields 中的 key 对应。
    pub fields: Value,
}

/// 更新文档请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct UpdateDocumentRequest {
    /// 新的父文档 ID。
    pub parent_id: Option<String>,
    /// 新的业务状态。
    pub status: Option<String>,
    /// 新的启用状态。
    pub enabled: Option<bool>,
    /// 需要更新的字段。
    /// 未传的字段保持原值不变。
    pub fields: Option<Value>,
}

/// 首次创建文档请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateLocalizedDocumentRequest {
    /// 当前新增页面选择的语言，例如 `zh_cn`。
    pub locale: String,
    /// 主记录初始状态。
    pub root_status: Option<String>,
    /// 主记录初始启用状态。
    pub root_enabled: Option<bool>,
    /// 主记录字段。
    /// 这里放跨语言共享的字段，例如 `code`、`sort_order`。
    pub root_fields: Option<Value>,
    /// 当前语言子记录初始状态。
    pub localized_status: Option<String>,
    /// 当前语言子记录初始启用状态。
    pub localized_enabled: Option<bool>,
    /// 当前语言子记录字段。
    /// 这里放随语言变化的字段，例如 `name`、`summary`、`body`。
    pub fields: Value,
}

/// 首次创建文档响应。
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateLocalizedDocumentResponse {
    /// 自动创建出来的主记录。
    pub root_document: DocumentView,
    /// 当前语言下的首条子记录。
    pub localized_document: DocumentView,
}

/// 新增某个语言版本的请求。
#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct CreateDocumentLocaleRequest {
    /// 要新增的语言，例如 `en_us`。
    pub locale: String,
    /// 新语言子记录初始状态。
    pub localized_status: Option<String>,
    /// 新语言子记录初始启用状态。
    pub localized_enabled: Option<bool>,
    /// 新语言子记录字段。
    pub fields: Value,
}

/// 语言优先的文档视图。
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct EntryView {
    /// 当前返回内容所对应的语言。
    pub locale: String,
    /// 当前主记录关联的翻译 schema key。
    pub translation_schema_key: String,
    /// 主记录。
    pub root_document: DocumentView,
    /// 当前语言对应的子记录。
    /// 如果该语言还没创建，这里会是 `null`。
    pub localized_document: Option<DocumentView>,
    /// 当前主记录已经存在的语言列表。
    pub available_locales: Vec<String>,
}

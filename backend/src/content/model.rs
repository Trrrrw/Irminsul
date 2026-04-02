use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

/// 支持的多语言代码。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Locale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "zh-TW")]
    ZhTw,
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "ja-JP")]
    JaJp,
}

impl Locale {
    /// 返回当前系统第一版正式支持的全部语言。
    pub const fn all() -> [Self; 4] {
        [Self::ZhCn, Self::ZhTw, Self::EnUs, Self::JaJp]
    }

    /// 返回写入数据库时使用的 locale 字符串。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::ZhTw => "zh-TW",
            Self::EnUs => "en-US",
            Self::JaJp => "ja-JP",
        }
    }
}

impl core::str::FromStr for Locale {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "zh-CN" => Ok(Self::ZhCn),
            "zh-TW" => Ok(Self::ZhTw),
            "en-US" => Ok(Self::EnUs),
            "ja-JP" => Ok(Self::JaJp),
            _ => Err("unsupported_locale"),
        }
    }
}

/// 通用内容实例的生命周期状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContentItemStatus {
    Draft,
    Published,
    Archived,
}

impl ContentItemStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Archived => "archived",
        }
    }
}

impl core::str::FromStr for ContentItemStatus {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            "archived" => Ok(Self::Archived),
            _ => Err("invalid_content_item_status"),
        }
    }
}

/// 向量任务执行状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl EmbeddingJobStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl core::str::FromStr for EmbeddingJobStatus {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err("invalid_embedding_job_status"),
        }
    }
}

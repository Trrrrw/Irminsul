use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
static RUNTIME_CONFIG: OnceLock<RwLock<AppConfig>> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub embedding: EmbeddingConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub default_provider: Option<String>,
    pub current_model: String,
    pub providers: Vec<EmbeddingProviderConfig>,
    pub api_keys: Vec<EmbeddingApiKeyConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingProviderConfig {
    pub code: String,
    pub name: String,
    pub base_url: String,
    pub embeddings_path: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingApiKeyConfig {
    pub provider_code: String,
    pub name: String,
    pub api_key: String,
    pub enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            embedding: EmbeddingConfig {
                default_provider: None,
                current_model: "text-embedding-3-small".to_string(),
                providers: Vec::new(),
                api_keys: Vec::new(),
            },
        }
    }
}

pub async fn init<P: AsRef<Path>>(data_dir: P) {
    let data_dir = data_dir.as_ref();
    fs::create_dir_all(data_dir).expect("failed to create data directory");

    let config_path = data_dir.join("config.toml");
    let config = if config_path.exists() {
        load_from_path(&config_path).expect("config.toml should load")
    } else {
        let config = AppConfig::default();
        write_config_file(&config_path, &config).expect("config.toml should be created");
        config
    };

    CONFIG_PATH
        .set(config_path)
        .expect("config path should be initialized once");
    RUNTIME_CONFIG
        .set(RwLock::new(config))
        .expect("runtime config should be initialized once");
}

pub async fn snapshot() -> AppConfig {
    RUNTIME_CONFIG
        .get()
        .expect("runtime config should be initialized")
        .read()
        .await
        .clone()
}

pub async fn update<F>(updater: F) -> Result<AppConfig, String>
where
    F: FnOnce(&mut AppConfig) -> Result<(), String>,
{
    let lock = RUNTIME_CONFIG
        .get()
        .expect("runtime config should be initialized");
    let mut guard = lock.write().await;
    let mut next = guard.clone();
    updater(&mut next)?;
    validate(&next)?;

    let path = CONFIG_PATH
        .get()
        .expect("config path should be initialized")
        .clone();
    write_config_file(&path, &next).map_err(|error| format!("写入 config.toml 失败: {error}"))?;
    *guard = next.clone();
    Ok(next)
}

fn load_from_path(path: &Path) -> Result<AppConfig, String> {
    let raw =
        fs::read_to_string(path).map_err(|error| format!("读取 config.toml 失败: {error}"))?;
    let config: AppConfig =
        toml::from_str(&raw).map_err(|error| format!("解析 config.toml 失败: {error}"))?;
    validate(&config)?;
    Ok(config)
}

fn validate(config: &AppConfig) -> Result<(), String> {
    if config.embedding.current_model.trim().is_empty() {
        return Err("embedding.current_model 不能为空".to_string());
    }

    let mut provider_codes = std::collections::BTreeSet::new();
    for provider in &config.embedding.providers {
        let code = provider.code.trim();
        if code.is_empty() {
            return Err("embedding provider code 不能为空".to_string());
        }
        if !provider_codes.insert(code.to_string()) {
            return Err(format!("embedding provider code 重复: {code}"));
        }
    }

    if let Some(default_provider) = &config.embedding.default_provider
        && !provider_codes.contains(default_provider)
    {
        return Err("default_provider 必须指向已存在的 provider".to_string());
    }

    for api_key in &config.embedding.api_keys {
        if !provider_codes.contains(api_key.provider_code.trim()) {
            return Err(format!(
                "embedding api key 关联了不存在的 provider: {}",
                api_key.provider_code
            ));
        }
        if api_key.name.trim().is_empty() {
            return Err("embedding api key name 不能为空".to_string());
        }
        if api_key.api_key.trim().is_empty() {
            return Err("embedding api key 不能为空".to_string());
        }
    }

    Ok(())
}

fn write_config_file(path: &Path, config: &AppConfig) -> Result<(), std::io::Error> {
    let serialized = toml::to_string_pretty(config).expect("config should serialize");
    let temp_path = path.with_extension("toml.tmp");
    fs::write(&temp_path, serialized)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temp_path, path)?;
    Ok(())
}

use crate::{
    admin::{
        dto::embedding::{
            AdminSettingsView, EmbeddingApiKeyInput, EmbeddingApiKeyView, EmbeddingProviderInput,
            EmbeddingProviderView, EmbeddingSettingsSectionView, EmbeddingSettingsView,
            UpdateAdminSettingsRequest,
        },
        services::audit::write_audit_log,
    },
    config::{self, AppConfig, EmbeddingApiKeyConfig, EmbeddingProviderConfig},
};

/// 运行时实际用于生成向量的配置快照。
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RuntimeEmbeddingConfig {
    pub base_url: String,
    pub embeddings_path: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Clone, Debug)]
pub struct UpdateSettingsResult {
    pub settings: AdminSettingsView,
    pub changed_model: bool,
    pub changed_fields: Vec<String>,
}

pub async fn get_settings_bundle() -> Result<AdminSettingsView, &'static str> {
    Ok(map_admin_settings_view(&config::snapshot().await))
}

pub async fn update_settings(
    payload: UpdateAdminSettingsRequest,
    actor_user_id: i64,
    actor_label: String,
    ip: Option<String>,
    user_agent: Option<String>,
) -> Result<UpdateSettingsResult, &'static str> {
    let before = config::snapshot().await;
    let updated = config::update(|config| apply_settings_update(config, payload.clone()))
        .await
        .map_err(|_| "settings_update_failed")?;
    let changed_model = before.embedding.current_model != updated.embedding.current_model;

    let mut changed_fields = Vec::new();
    if before.embedding.default_provider != updated.embedding.default_provider {
        changed_fields.push("embedding.default_provider".to_string());
    }
    if before.embedding.current_model != updated.embedding.current_model {
        changed_fields.push("embedding.current_model".to_string());
    }
    if before.embedding.providers != updated.embedding.providers {
        changed_fields.push("embedding.providers".to_string());
    }
    if before.embedding.api_keys != updated.embedding.api_keys {
        changed_fields.push("embedding.api_keys".to_string());
    }

    write_audit_log(
        Some(actor_user_id),
        Some(actor_label),
        "update_admin_settings",
        "system_config",
        Some("embedding".to_string()),
        "updated config.toml settings",
        Some(serde_json::json!({
            "changed_fields": changed_fields,
            "changed_model": changed_model,
        })),
        ip,
        user_agent,
    )
    .await;

    Ok(UpdateSettingsResult {
        settings: map_admin_settings_view(&updated),
        changed_model,
        changed_fields,
    })
}

#[allow(dead_code)]
pub async fn get_runtime_embedding_config() -> Result<RuntimeEmbeddingConfig, String> {
    let config = config::snapshot().await;
    let provider_code = config
        .embedding
        .default_provider
        .clone()
        .ok_or_else(|| "未配置默认 embedding provider".to_string())?;
    let provider = config
        .embedding
        .providers
        .iter()
        .find(|provider| provider.code == provider_code && provider.enabled)
        .ok_or_else(|| "默认 embedding provider 不存在或已禁用".to_string())?;
    let api_key = config
        .embedding
        .api_keys
        .iter()
        .find(|api_key| api_key.provider_code == provider.code && api_key.enabled)
        .ok_or_else(|| "没有可用的 embedding API key".to_string())?;

    Ok(RuntimeEmbeddingConfig {
        base_url: provider.base_url.clone(),
        embeddings_path: provider.embeddings_path.clone(),
        api_key: api_key.api_key.clone(),
        model: config.embedding.current_model,
    })
}

fn apply_settings_update(
    config: &mut AppConfig,
    payload: UpdateAdminSettingsRequest,
) -> Result<(), String> {
    let Some(embedding) = payload.embedding else {
        return Ok(());
    };

    if let Some(providers) = embedding.providers {
        config.embedding.providers = providers
            .into_iter()
            .map(map_provider_input)
            .collect::<Result<Vec<_>, _>>()?;
    }

    if let Some(api_keys) = embedding.api_keys {
        let current_keys = config.embedding.api_keys.clone();
        config.embedding.api_keys = api_keys
            .into_iter()
            .map(|input| map_api_key_input(input, &current_keys))
            .collect::<Result<Vec<_>, _>>()?;
    }

    if let Some(settings) = embedding.settings {
        config.embedding.default_provider = settings.default_provider.map(normalize_code);
        config.embedding.current_model =
            normalize_required_text(&settings.current_model, "embedding_model_required")?;
    }

    Ok(())
}

fn map_admin_settings_view(config: &AppConfig) -> AdminSettingsView {
    AdminSettingsView {
        embedding: EmbeddingSettingsSectionView {
            settings: EmbeddingSettingsView {
                default_provider: config.embedding.default_provider.clone(),
                current_model: config.embedding.current_model.clone(),
                updated_at: crate::admin::middlewares::auth::unix_timestamp(),
            },
            providers: config
                .embedding
                .providers
                .iter()
                .cloned()
                .map(|provider| EmbeddingProviderView {
                    code: provider.code,
                    name: provider.name,
                    base_url: provider.base_url,
                    embeddings_path: provider.embeddings_path,
                    enabled: provider.enabled,
                })
                .collect(),
            api_keys: config
                .embedding
                .api_keys
                .iter()
                .cloned()
                .map(|api_key| EmbeddingApiKeyView {
                    id: api_key_identity(&api_key.provider_code, &api_key.name),
                    provider_code: api_key.provider_code,
                    name: api_key.name,
                    masked_api_key: mask_api_key(&api_key.api_key),
                    enabled: api_key.enabled,
                })
                .collect(),
        },
    }
}

fn map_provider_input(payload: EmbeddingProviderInput) -> Result<EmbeddingProviderConfig, String> {
    Ok(EmbeddingProviderConfig {
        code: normalize_code(payload.code),
        name: normalize_required_text(&payload.name, "embedding_provider_name_required")?,
        base_url: normalize_required_text(
            &payload.base_url,
            "embedding_provider_base_url_required",
        )?,
        embeddings_path: payload
            .embeddings_path
            .map(normalize_required_path)
            .unwrap_or_else(|| "/embeddings".to_string()),
        enabled: payload.enabled,
    })
}

fn map_api_key_input(
    payload: EmbeddingApiKeyInput,
    current_keys: &[EmbeddingApiKeyConfig],
) -> Result<EmbeddingApiKeyConfig, String> {
    let provider_code = normalize_code(payload.provider_code);
    let name = normalize_required_text(&payload.name, "embedding_api_key_name_required")?;
    let api_key = match payload.api_key {
        Some(value) => normalize_required_text(&value, "embedding_api_key_required")?,
        None => current_keys
            .iter()
            .find(|candidate| {
                api_key_identity(&candidate.provider_code, &candidate.name)
                    == payload.id.clone().unwrap_or_default()
            })
            .map(|candidate| candidate.api_key.clone())
            .ok_or_else(|| "embedding_api_key_required".to_string())?,
    };

    Ok(EmbeddingApiKeyConfig {
        provider_code,
        name,
        api_key,
        enabled: payload.enabled,
    })
}

fn api_key_identity(provider_code: &str, name: &str) -> String {
    format!("{provider_code}:{name}")
}

fn mask_api_key(value: &str) -> String {
    if value.len() <= 8 {
        return "********".to_string();
    }
    format!("{}****{}", &value[..4], &value[value.len() - 4..])
}

fn normalize_code(value: String) -> String {
    value.trim().to_ascii_lowercase().replace(' ', "_")
}

fn normalize_required_text(value: &str, error: &str) -> Result<String, String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(error.to_string());
    }
    Ok(normalized)
}

fn normalize_required_path(value: String) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "/embeddings".to_string()
    } else {
        trimmed.to_string()
    }
}

use crate::admin::{
    db,
    dto::embedding::{
        AdminSettingsView, EmbeddingApiKeyInput, EmbeddingApiKeyView, EmbeddingProviderInput,
        EmbeddingProviderView, EmbeddingSettingsSectionView, EmbeddingSettingsView,
        UpdateEmbeddingSettingsRequest,
    },
    entities::{embedding_api_keys, embedding_providers, embedding_settings},
    services::audit::write_audit_log,
};

/// 运行时实际用于生成向量的配置快照。
#[derive(Clone, Debug)]
pub struct RuntimeEmbeddingConfig {
    pub base_url: String,
    pub embeddings_path: String,
    pub api_key: String,
    pub model: String,
}

/// 列出全部 embedding 提供方。
pub async fn list_providers() -> Vec<EmbeddingProviderView> {
    let Ok(conn) = db::database().connect() else {
        return Vec::new();
    };
    let Ok(mut rows) = conn
        .query(
            "SELECT id, code, name, base_url, embeddings_path, enabled, created_at, updated_at
             FROM EMBEDDING_PROVIDERS ORDER BY id ASC",
            (),
        )
        .await
    else {
        return Vec::new();
    };

    let mut values = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        if let Ok(provider) = map_provider_row(&row) {
            values.push(map_provider_view(provider));
        }
    }
    values
}

/// 在设置页中创建或更新 embedding 提供方。
pub async fn upsert_provider(
    payload: EmbeddingProviderInput,
) -> Result<EmbeddingProviderView, &'static str> {
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database().connect().map_err(|_| "db_unavailable")?;
    let normalized_code = normalize_code(&payload.code)?;
    let normalized_name =
        normalize_required_text(&payload.name, "embedding_provider_name_required")?;
    let normalized_base_url =
        normalize_required_text(&payload.base_url, "embedding_provider_base_url_required")?;
    let normalized_path = payload
        .embeddings_path
        .unwrap_or_else(|| "/embeddings".to_string());
    let enabled = if payload.enabled { 1 } else { 0 };

    if let Some(id) = payload.id {
        find_provider_by_id(&conn, id)
            .await
            .ok_or("embedding_provider_not_found")?;
        conn.execute(
            "UPDATE EMBEDDING_PROVIDERS
             SET code = ?1, name = ?2, base_url = ?3, embeddings_path = ?4, enabled = ?5, updated_at = ?6
             WHERE id = ?7",
            turso::params![
                normalized_code,
                normalized_name,
                normalized_base_url,
                normalized_path,
                enabled,
                now,
                id,
            ],
        )
        .await
        .map_err(|_| "embedding_provider_update_failed")?;
        let provider = find_provider_by_id(&conn, id)
            .await
            .ok_or("embedding_provider_update_failed")?;
        return Ok(map_provider_view(provider));
    }

    conn.execute(
        "INSERT INTO EMBEDDING_PROVIDERS
         (code, name, base_url, embeddings_path, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        turso::params![
            normalized_code,
            normalized_name,
            normalized_base_url,
            normalized_path,
            enabled,
            now,
        ],
    )
    .await
    .map_err(|_| "embedding_provider_create_failed")?;
    let provider = find_provider_by_id(&conn, conn.last_insert_rowid())
        .await
        .ok_or("embedding_provider_create_failed")?;
    Ok(map_provider_view(provider))
}

/// 列出 API key。
pub async fn list_api_keys(provider_id: Option<i64>) -> Vec<EmbeddingApiKeyView> {
    let Ok(conn) = db::database().connect() else {
        return Vec::new();
    };
    let query = if provider_id.is_some() {
        "SELECT id, provider_id, name, api_key, enabled, created_at, updated_at
         FROM EMBEDDING_API_KEYS WHERE provider_id = ?1 ORDER BY id ASC"
    } else {
        "SELECT id, provider_id, name, api_key, enabled, created_at, updated_at
         FROM EMBEDDING_API_KEYS ORDER BY id ASC"
    };
    let rows_result = if let Some(provider_id) = provider_id {
        conn.query(query, turso::params![provider_id]).await
    } else {
        conn.query(query, ()).await
    };
    let Ok(mut rows) = rows_result else {
        return Vec::new();
    };

    let mut values = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        if let Ok(key) = map_api_key_row(&row) {
            values.push(map_api_key_view(key));
        }
    }
    values
}

/// 在设置页中创建或更新 embedding API key。
pub async fn upsert_api_key(
    payload: EmbeddingApiKeyInput,
) -> Result<EmbeddingApiKeyView, &'static str> {
    let conn = db::database().connect().map_err(|_| "db_unavailable")?;
    if find_provider_by_id(&conn, payload.provider_id)
        .await
        .is_none()
    {
        return Err("embedding_provider_not_found");
    }

    let now = crate::admin::middlewares::auth::unix_timestamp();
    let normalized_name =
        normalize_required_text(&payload.name, "embedding_api_key_name_required")?;
    let enabled = if payload.enabled { 1 } else { 0 };

    if let Some(id) = payload.id {
        let current = find_api_key_by_id(&conn, id)
            .await
            .ok_or("embedding_api_key_not_found")?;
        let api_key = match payload.api_key {
            Some(value) => normalize_required_text(&value, "embedding_api_key_required")?,
            None => current.api_key,
        };
        conn.execute(
            "UPDATE EMBEDDING_API_KEYS
             SET provider_id = ?1, name = ?2, api_key = ?3, enabled = ?4, updated_at = ?5
             WHERE id = ?6",
            turso::params![
                payload.provider_id,
                normalized_name,
                api_key,
                enabled,
                now,
                id
            ],
        )
        .await
        .map_err(|_| "embedding_api_key_update_failed")?;
        let key = find_api_key_by_id(&conn, id)
            .await
            .ok_or("embedding_api_key_update_failed")?;
        return Ok(map_api_key_view(key));
    }

    let api_key = match payload.api_key {
        Some(value) => normalize_required_text(&value, "embedding_api_key_required")?,
        None => return Err("embedding_api_key_required"),
    };
    conn.execute(
        "INSERT INTO EMBEDDING_API_KEYS
         (provider_id, name, api_key, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        turso::params![payload.provider_id, normalized_name, api_key, enabled, now],
    )
    .await
    .map_err(|_| "embedding_api_key_create_failed")?;
    let key = find_api_key_by_id(&conn, conn.last_insert_rowid())
        .await
        .ok_or("embedding_api_key_create_failed")?;
    Ok(map_api_key_view(key))
}

/// 返回全局 embedding 设置。
pub async fn get_settings() -> Result<EmbeddingSettingsView, &'static str> {
    let settings = ensure_embedding_settings().await?;
    Ok(EmbeddingSettingsView {
        default_provider_id: settings.default_provider_id,
        current_model: settings.current_model,
        updated_at: settings.updated_at,
    })
}

/// 返回设置页需要的完整 embedding 聚合数据。
pub async fn get_settings_bundle() -> Result<AdminSettingsView, &'static str> {
    Ok(AdminSettingsView {
        embedding: EmbeddingSettingsSectionView {
            settings: get_settings().await?,
            providers: list_providers().await,
            api_keys: list_api_keys(None).await,
        },
    })
}

/// 更新全局 embedding 设置。
pub async fn update_settings(
    payload: UpdateEmbeddingSettingsRequest,
    actor_user_id: i64,
    actor_label: String,
    ip: Option<String>,
    user_agent: Option<String>,
) -> Result<(EmbeddingSettingsView, bool), &'static str> {
    let conn = db::database().connect().map_err(|_| "db_unavailable")?;
    if let Some(provider_id) = payload.default_provider_id
        && find_provider_by_id(&conn, provider_id).await.is_none()
    {
        return Err("embedding_provider_not_found");
    }

    let current = ensure_embedding_settings().await?;
    let next_model = normalize_required_text(&payload.current_model, "embedding_model_required")?;
    let changed_model = current.current_model != next_model;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    conn.execute(
        "UPDATE EMBEDDING_SETTINGS
         SET default_provider_id = ?1, current_model = ?2, updated_at = ?3
         WHERE id = 1",
        turso::params![payload.default_provider_id, next_model.clone(), now],
    )
    .await
    .map_err(|_| "embedding_settings_update_failed")?;

    write_audit_log(
        Some(actor_user_id),
        Some(actor_label),
        "update_embedding_settings",
        "embedding_settings",
        Some("1".to_string()),
        "updated embedding settings",
        Some(serde_json::json!({
            "default_provider_id": payload.default_provider_id,
            "current_model": next_model,
            "changed_model": changed_model,
        })),
        ip,
        user_agent,
    )
    .await;

    Ok((
        EmbeddingSettingsView {
            default_provider_id: payload.default_provider_id,
            current_model: next_model,
            updated_at: now,
        },
        changed_model,
    ))
}

/// 返回当前运行时有效的 embedding 配置。
pub async fn get_runtime_embedding_config() -> Result<RuntimeEmbeddingConfig, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开后台数据库失败: {error}"))?;
    let settings = ensure_embedding_settings()
        .await
        .map_err(|error| error.to_string())?;
    let provider_id = settings
        .default_provider_id
        .ok_or_else(|| "未配置默认 embedding provider".to_string())?;
    let provider = find_provider_by_id(&conn, provider_id)
        .await
        .ok_or_else(|| "默认 embedding provider 不存在".to_string())?;
    if !provider.enabled {
        return Err("默认 embedding provider 不存在或已禁用".to_string());
    }
    let api_key = find_first_enabled_api_key(&conn, provider.id)
        .await
        .ok_or_else(|| "没有可用的 embedding API key".to_string())?;

    Ok(RuntimeEmbeddingConfig {
        base_url: provider.base_url,
        embeddings_path: provider.embeddings_path,
        api_key: api_key.api_key,
        model: settings.current_model,
    })
}

/// 确保系统存在一条默认设置记录。
pub async fn ensure_embedding_settings() -> Result<embedding_settings::Model, &'static str> {
    let conn = db::database().connect().map_err(|_| "db_unavailable")?;
    if let Some(settings) = find_settings(&conn).await {
        return Ok(settings);
    }

    let now = crate::admin::middlewares::auth::unix_timestamp();
    conn.execute(
        "INSERT INTO EMBEDDING_SETTINGS (id, default_provider_id, current_model, updated_at)
         VALUES (1, NULL, ?1, ?2)",
        turso::params!["text-embedding-3-small".to_string(), now],
    )
    .await
    .map_err(|_| "embedding_settings_init_failed")?;
    find_settings(&conn)
        .await
        .ok_or("embedding_settings_init_failed")
}

async fn find_provider_by_id(
    conn: &turso::Connection,
    id: i64,
) -> Option<embedding_providers::Model> {
    let mut rows = conn
        .query(
            "SELECT id, code, name, base_url, embeddings_path, enabled, created_at, updated_at
             FROM EMBEDDING_PROVIDERS WHERE id = ?1 LIMIT 1",
            turso::params![id],
        )
        .await
        .ok()?;
    let row = rows.next().await.ok().flatten()?;
    map_provider_row(&row).ok()
}

async fn find_api_key_by_id(
    conn: &turso::Connection,
    id: i64,
) -> Option<embedding_api_keys::Model> {
    let mut rows = conn
        .query(
            "SELECT id, provider_id, name, api_key, enabled, created_at, updated_at
             FROM EMBEDDING_API_KEYS WHERE id = ?1 LIMIT 1",
            turso::params![id],
        )
        .await
        .ok()?;
    let row = rows.next().await.ok().flatten()?;
    map_api_key_row(&row).ok()
}

async fn find_first_enabled_api_key(
    conn: &turso::Connection,
    provider_id: i64,
) -> Option<embedding_api_keys::Model> {
    let mut rows = conn
        .query(
            "SELECT id, provider_id, name, api_key, enabled, created_at, updated_at
             FROM EMBEDDING_API_KEYS
             WHERE provider_id = ?1 AND enabled = 1
             ORDER BY id ASC
             LIMIT 1",
            turso::params![provider_id],
        )
        .await
        .ok()?;
    let row = rows.next().await.ok().flatten()?;
    map_api_key_row(&row).ok()
}

async fn find_settings(conn: &turso::Connection) -> Option<embedding_settings::Model> {
    let mut rows = conn
        .query(
            "SELECT id, default_provider_id, current_model, updated_at
             FROM EMBEDDING_SETTINGS WHERE id = 1 LIMIT 1",
            (),
        )
        .await
        .ok()?;
    let row = rows.next().await.ok().flatten()?;
    map_settings_row(&row).ok()
}

fn map_provider_row(row: &turso::Row) -> Result<embedding_providers::Model, String> {
    Ok(embedding_providers::Model {
        id: row.get(0).map_err(|error| error.to_string())?,
        code: row.get(1).map_err(|error| error.to_string())?,
        name: row.get(2).map_err(|error| error.to_string())?,
        base_url: row.get(3).map_err(|error| error.to_string())?,
        embeddings_path: row.get(4).map_err(|error| error.to_string())?,
        enabled: row.get::<i64>(5).map_err(|error| error.to_string())? != 0,
        created_at: row.get(6).map_err(|error| error.to_string())?,
        updated_at: row.get(7).map_err(|error| error.to_string())?,
    })
}

fn map_api_key_row(row: &turso::Row) -> Result<embedding_api_keys::Model, String> {
    Ok(embedding_api_keys::Model {
        id: row.get(0).map_err(|error| error.to_string())?,
        provider_id: row.get(1).map_err(|error| error.to_string())?,
        name: row.get(2).map_err(|error| error.to_string())?,
        api_key: row.get(3).map_err(|error| error.to_string())?,
        enabled: row.get::<i64>(4).map_err(|error| error.to_string())? != 0,
        created_at: row.get(5).map_err(|error| error.to_string())?,
        updated_at: row.get(6).map_err(|error| error.to_string())?,
    })
}

fn map_settings_row(row: &turso::Row) -> Result<embedding_settings::Model, String> {
    Ok(embedding_settings::Model {
        id: row.get(0).map_err(|error| error.to_string())?,
        default_provider_id: row.get(1).map_err(|error| error.to_string())?,
        current_model: row.get(2).map_err(|error| error.to_string())?,
        updated_at: row.get(3).map_err(|error| error.to_string())?,
    })
}

fn map_provider_view(provider: embedding_providers::Model) -> EmbeddingProviderView {
    EmbeddingProviderView {
        id: provider.id,
        code: provider.code,
        name: provider.name,
        base_url: provider.base_url,
        embeddings_path: provider.embeddings_path,
        enabled: provider.enabled,
        created_at: provider.created_at,
        updated_at: provider.updated_at,
    }
}

fn map_api_key_view(key: embedding_api_keys::Model) -> EmbeddingApiKeyView {
    EmbeddingApiKeyView {
        id: key.id,
        provider_id: key.provider_id,
        name: key.name,
        masked_api_key: mask_api_key(&key.api_key),
        enabled: key.enabled,
        created_at: key.created_at,
        updated_at: key.updated_at,
    }
}

fn mask_api_key(value: &str) -> String {
    if value.len() <= 8 {
        return "********".to_string();
    }
    format!("{}****{}", &value[..4], &value[value.len() - 4..])
}

fn normalize_code(value: &str) -> Result<String, &'static str> {
    let normalized = value.trim().to_ascii_lowercase().replace(' ', "_");
    if normalized.is_empty() {
        return Err("embedding_provider_code_required");
    }
    Ok(normalized)
}

fn normalize_required_text(value: &str, error: &'static str) -> Result<String, &'static str> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(error);
    }
    Ok(normalized)
}

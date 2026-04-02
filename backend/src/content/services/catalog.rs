use crate::content::{
    db,
    dto::content::{
        ContentItemDetailView, ContentItemTextView, ContentItemView, ContentTypeMetadataView,
        ContentTypeTextView, ContentTypeView, CreateContentItemRequest, CreateContentTypeRequest,
        CreateGameRequest, EmbeddingJobView, GameDetailView, GameTextView, GameView,
        SetEnabledRequest, UpdateContentItemRequest, UpdateContentTypeRequest, UpdateGameRequest,
        UpsertContentItemTextRequest, UpsertContentTypeTextRequest, UpsertGameTextRequest,
    },
    model::{ContentItemStatus, EmbeddingJobStatus, Locale},
};

/// 返回固定核心字段列表，供 panel 元数据接口驱动表单。
pub fn core_fields() -> Vec<String> {
    vec![
        "name".to_string(),
        "subtitle".to_string(),
        "author".to_string(),
        "summary".to_string(),
        "body".to_string(),
    ]
}

/// 返回系统当前支持的全部语言。
pub fn supported_locales() -> Vec<String> {
    Locale::all()
        .into_iter()
        .map(|locale| locale.as_str().to_string())
        .collect()
}

/// 列出全部游戏。
pub async fn list_games(locale: Option<&str>) -> Result<Vec<GameView>, String> {
    let preferred_locale = parse_optional_locale(locale)?;
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut rows = conn
        .query(
            "SELECT id, code, enabled, created_at, updated_at FROM games ORDER BY id ASC",
            (),
        )
        .await
        .map_err(|error| format!("查询游戏列表失败: {error}"))?;

    let mut values = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取游戏列表失败: {error}"))?
    {
        let mut game = map_game_row(&row)?;
        apply_game_display_text(&mut game, preferred_locale).await;
        values.push(game);
    }
    Ok(values)
}

/// 创建游戏。
pub async fn create_game(payload: CreateGameRequest) -> Result<GameView, &'static str> {
    let code = normalize_code(&payload.code)?;
    let name = normalize_required_text(&payload.name, "game_name_required")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;

    conn.execute(
        "INSERT INTO games (code, name, description, enabled, created_at, updated_at) VALUES (?1, '', NULL, 1, ?2, ?2)",
        turso::params![code, now],
    )
    .await
    .map_err(|_| "game_create_failed")?;

    let game_id = last_insert_rowid(&conn)
        .await
        .map_err(|_| "game_create_failed")?;
    upsert_game_text(
        game_id,
        payload.locale,
        UpsertGameTextRequest {
            name,
            description: payload.description,
        },
    )
    .await
    .map_err(|_| "game_create_failed")?;
    find_game(game_id, Some(payload.locale))
        .await
        .ok_or("game_create_failed")
}

/// 更新游戏。
pub async fn update_game(id: i64, payload: UpdateGameRequest) -> Result<GameView, &'static str> {
    let existing = find_game(id, None).await.ok_or("game_not_found")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "UPDATE games SET code = ?1, updated_at = ?2 WHERE id = ?3",
        turso::params![
            normalize_code(payload.code.as_deref().unwrap_or(&existing.code))?,
            now,
            id
        ],
    )
    .await
    .map_err(|_| "game_update_failed")?;
    find_game(id, None).await.ok_or("game_update_failed")
}

/// 更新游戏启用状态。
pub async fn set_game_enabled(
    id: i64,
    payload: SetEnabledRequest,
) -> Result<GameView, &'static str> {
    find_game(id, None).await.ok_or("game_not_found")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "UPDATE games SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
        turso::params![bool_to_i64(payload.enabled), now, id],
    )
    .await
    .map_err(|_| "game_update_failed")?;
    find_game(id, None).await.ok_or("game_update_failed")
}

/// 查找单个游戏。
pub async fn find_game(id: i64, locale: Option<Locale>) -> Option<GameView> {
    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, code, enabled, created_at, updated_at FROM games WHERE id = ?1",
            turso::params![id],
        )
        .await
        .ok()?;
    let mut game = rows
        .next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_game_row(&row).ok())?;
    apply_game_display_text(&mut game, locale).await;
    Some(game)
}

/// 返回单个游戏详情。
pub async fn get_game_detail(
    id: i64,
    locale: Option<&str>,
) -> Result<GameDetailView, &'static str> {
    let preferred_locale = parse_optional_locale(locale).map_err(|_| "unsupported_locale")?;
    let game = find_game(id, preferred_locale)
        .await
        .ok_or("game_not_found")?;
    let localized_texts = list_game_texts(id)
        .await
        .map_err(|_| "game_detail_failed")?;
    let missing_locales = supported_locales()
        .into_iter()
        .filter(|value| {
            !localized_texts
                .iter()
                .any(|text| text.locale.as_str() == value.as_str())
        })
        .collect();
    Ok(GameDetailView {
        game,
        localized_texts,
        missing_locales,
    })
}

/// 列出某个游戏的全部语言文本。
pub async fn list_game_texts(game_id: i64) -> Result<Vec<GameTextView>, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut rows = conn
        .query(
            "SELECT id, game_id, locale, name, description, created_at, updated_at
             FROM game_texts WHERE game_id = ?1 ORDER BY id ASC",
            turso::params![game_id],
        )
        .await
        .map_err(|error| format!("查询游戏文本失败: {error}"))?;

    let mut values = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取游戏文本失败: {error}"))?
    {
        values.push(map_game_text_row(&row)?);
    }
    Ok(values)
}

/// 新增或更新某个游戏的指定语言文本。
pub async fn upsert_game_text(
    game_id: i64,
    locale: Locale,
    payload: UpsertGameTextRequest,
) -> Result<GameTextView, &'static str> {
    find_game(game_id, None).await.ok_or("game_not_found")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    let existing = find_game_text(game_id, locale).await;

    if let Some(existing) = existing {
        conn.execute(
            "UPDATE game_texts SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            turso::params![
                normalize_required_text(&payload.name, "game_name_required")?,
                normalize_optional_text(payload.description),
                now,
                existing.id,
            ],
        )
        .await
        .map_err(|_| "game_text_update_failed")?;
    } else {
        conn.execute(
            "INSERT INTO game_texts (game_id, locale, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            turso::params![
                game_id,
                locale.as_str(),
                normalize_required_text(&payload.name, "game_name_required")?,
                normalize_optional_text(payload.description),
                now,
            ],
        )
        .await
        .map_err(|_| "game_text_create_failed")?;
    }

    find_game_text(game_id, locale)
        .await
        .ok_or("game_text_create_failed")
}

/// 删除某个游戏的指定语言文本。
pub async fn delete_game_text(game_id: i64, locale: &str) -> Result<(), &'static str> {
    let locale = locale.parse::<Locale>().map_err(|_| "unsupported_locale")?;
    let text = find_game_text(game_id, locale)
        .await
        .ok_or("game_text_not_found")?;
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "DELETE FROM game_texts WHERE id = ?1",
        turso::params![text.id],
    )
    .await
    .map_err(|_| "game_text_delete_failed")?;
    Ok(())
}

/// 列出内容类型。
pub async fn list_content_types(
    game_id: Option<i64>,
    locale: Option<&str>,
) -> Result<Vec<ContentTypeView>, String> {
    let preferred_locale = parse_optional_locale(locale)?;
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let sql = if game_id.is_some() {
        "SELECT id, game_id, code, name, description, supports_i18n, supports_embedding, enabled, created_at, updated_at FROM content_types WHERE game_id = ?1 ORDER BY id ASC"
    } else {
        "SELECT id, game_id, code, name, description, supports_i18n, supports_embedding, enabled, created_at, updated_at FROM content_types ORDER BY id ASC"
    };
    let mut rows = if let Some(game_id) = game_id {
        conn.query(sql, turso::params![game_id])
            .await
            .map_err(|error| format!("查询内容类型失败: {error}"))?
    } else {
        conn.query(sql, ())
            .await
            .map_err(|error| format!("查询内容类型失败: {error}"))?
    };

    let mut values = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取内容类型失败: {error}"))?
    {
        let mut content_type = map_content_type_row(&row)?;
        apply_content_type_display_text(&mut content_type, preferred_locale).await;
        values.push(content_type);
    }
    Ok(values)
}

/// 创建内容类型。
pub async fn create_content_type(
    payload: CreateContentTypeRequest,
) -> Result<ContentTypeView, &'static str> {
    ensure_game_exists(payload.game_id).await?;
    let code = normalize_code(&payload.code)?;
    let name = normalize_required_text(&payload.name, "content_type_name_required")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;

    conn.execute(
        "INSERT INTO content_types (game_id, code, name, description, supports_i18n, supports_embedding, enabled, created_at, updated_at)
         VALUES (?1, ?2, '', NULL, ?3, ?4, 1, ?5, ?5)",
        turso::params![
            payload.game_id,
            code,
            bool_to_i64(payload.supports_i18n),
            bool_to_i64(payload.supports_embedding),
            now
        ],
    )
    .await
    .map_err(|_| "content_type_create_failed")?;

    let id = last_insert_rowid(&conn)
        .await
        .map_err(|_| "content_type_create_failed")?;
    upsert_content_type_text(
        id,
        payload.locale,
        UpsertContentTypeTextRequest {
            name,
            description: payload.description,
        },
    )
    .await
    .map_err(|_| "content_type_create_failed")?;
    find_content_type(id, Some(payload.locale))
        .await
        .ok_or("content_type_create_failed")
}

/// 更新内容类型。
pub async fn update_content_type(
    id: i64,
    payload: UpdateContentTypeRequest,
) -> Result<ContentTypeView, &'static str> {
    let existing = find_content_type(id, None)
        .await
        .ok_or("content_type_not_found")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "UPDATE content_types
         SET code = ?1, supports_i18n = ?2, supports_embedding = ?3, updated_at = ?4
         WHERE id = ?5",
        turso::params![
            normalize_code(payload.code.as_deref().unwrap_or(&existing.code))?,
            bool_to_i64(payload.supports_i18n.unwrap_or(existing.supports_i18n)),
            bool_to_i64(
                payload
                    .supports_embedding
                    .unwrap_or(existing.supports_embedding)
            ),
            now,
            id
        ],
    )
    .await
    .map_err(|_| "content_type_update_failed")?;
    find_content_type(id, None)
        .await
        .ok_or("content_type_update_failed")
}

/// 更新内容类型启用状态。
pub async fn set_content_type_enabled(
    id: i64,
    payload: SetEnabledRequest,
) -> Result<ContentTypeView, &'static str> {
    find_content_type(id, None)
        .await
        .ok_or("content_type_not_found")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "UPDATE content_types SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
        turso::params![bool_to_i64(payload.enabled), now, id],
    )
    .await
    .map_err(|_| "content_type_update_failed")?;
    find_content_type(id, None)
        .await
        .ok_or("content_type_update_failed")
}

/// 返回内容类型元信息。
pub async fn get_content_type_metadata(
    id: i64,
    locale: Option<&str>,
) -> Result<ContentTypeMetadataView, &'static str> {
    let preferred_locale = parse_optional_locale(locale).map_err(|_| "unsupported_locale")?;
    let content_type = find_content_type(id, preferred_locale)
        .await
        .ok_or("content_type_not_found")?;
    let localized_texts = list_content_type_texts(id)
        .await
        .map_err(|_| "content_type_metadata_failed")?;
    let missing_locales = supported_locales()
        .into_iter()
        .filter(|value| {
            !localized_texts
                .iter()
                .any(|text| text.locale.as_str() == value.as_str())
        })
        .collect();
    Ok(ContentTypeMetadataView {
        content_type,
        localized_texts,
        missing_locales,
        supported_locales: supported_locales(),
        core_fields: core_fields(),
    })
}

/// 查找单个内容类型。
pub async fn find_content_type(id: i64, locale: Option<Locale>) -> Option<ContentTypeView> {
    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, game_id, code, name, description, supports_i18n, supports_embedding, enabled, created_at, updated_at
             FROM content_types WHERE id = ?1",
            turso::params![id],
        )
        .await
        .ok()?;
    let mut content_type = rows
        .next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_content_type_row(&row).ok())?;
    apply_content_type_display_text(&mut content_type, locale).await;
    Some(content_type)
}

/// 列出某个内容类型的全部语言文本。
pub async fn list_content_type_texts(
    content_type_id: i64,
) -> Result<Vec<ContentTypeTextView>, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut rows = conn
        .query(
            "SELECT id, content_type_id, locale, name, description, created_at, updated_at
             FROM content_type_texts WHERE content_type_id = ?1 ORDER BY id ASC",
            turso::params![content_type_id],
        )
        .await
        .map_err(|error| format!("查询内容类型文本失败: {error}"))?;

    let mut values = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取内容类型文本失败: {error}"))?
    {
        values.push(map_content_type_text_row(&row)?);
    }
    Ok(values)
}

/// 新增或更新某个内容类型的指定语言文本。
pub async fn upsert_content_type_text(
    content_type_id: i64,
    locale: Locale,
    payload: UpsertContentTypeTextRequest,
) -> Result<ContentTypeTextView, &'static str> {
    find_content_type(content_type_id, None)
        .await
        .ok_or("content_type_not_found")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    let existing = find_content_type_text(content_type_id, locale).await;

    if let Some(existing) = existing {
        conn.execute(
            "UPDATE content_type_texts SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            turso::params![
                normalize_required_text(&payload.name, "content_type_name_required")?,
                normalize_optional_text(payload.description),
                now,
                existing.id,
            ],
        )
        .await
        .map_err(|_| "content_type_text_update_failed")?;
    } else {
        conn.execute(
            "INSERT INTO content_type_texts (content_type_id, locale, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            turso::params![
                content_type_id,
                locale.as_str(),
                normalize_required_text(&payload.name, "content_type_name_required")?,
                normalize_optional_text(payload.description),
                now,
            ],
        )
        .await
        .map_err(|_| "content_type_text_create_failed")?;
    }

    find_content_type_text(content_type_id, locale)
        .await
        .ok_or("content_type_text_create_failed")
}

/// 删除某个内容类型的指定语言文本。
pub async fn delete_content_type_text(
    content_type_id: i64,
    locale: &str,
) -> Result<(), &'static str> {
    let locale = locale.parse::<Locale>().map_err(|_| "unsupported_locale")?;
    let text = find_content_type_text(content_type_id, locale)
        .await
        .ok_or("content_type_text_not_found")?;
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "DELETE FROM content_type_texts WHERE id = ?1",
        turso::params![text.id],
    )
    .await
    .map_err(|_| "content_type_text_delete_failed")?;
    Ok(())
}

/// 列出通用内容实例。
pub async fn list_content_items(
    game_id: Option<i64>,
    content_type_id: Option<i64>,
    locale: Option<&str>,
) -> Result<Vec<ContentItemView>, String> {
    let preferred_locale = parse_optional_locale(locale)?;
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut values = Vec::new();

    let mut rows = match (game_id, content_type_id) {
        (Some(game_id), Some(content_type_id)) => conn
            .query(
                "SELECT id, game_id, content_type_id, slug, external_key, status, sort_order, enabled, created_at, updated_at
                 FROM content_items WHERE game_id = ?1 AND content_type_id = ?2 ORDER BY id ASC",
                turso::params![game_id, content_type_id],
            )
            .await
            .map_err(|error| format!("查询内容实例失败: {error}"))?,
        (Some(game_id), None) => conn
            .query(
                "SELECT id, game_id, content_type_id, slug, external_key, status, sort_order, enabled, created_at, updated_at
                 FROM content_items WHERE game_id = ?1 ORDER BY id ASC",
                turso::params![game_id],
            )
            .await
            .map_err(|error| format!("查询内容实例失败: {error}"))?,
        (None, Some(content_type_id)) => conn
            .query(
                "SELECT id, game_id, content_type_id, slug, external_key, status, sort_order, enabled, created_at, updated_at
                 FROM content_items WHERE content_type_id = ?1 ORDER BY id ASC",
                turso::params![content_type_id],
            )
            .await
            .map_err(|error| format!("查询内容实例失败: {error}"))?,
        (None, None) => conn
            .query(
                "SELECT id, game_id, content_type_id, slug, external_key, status, sort_order, enabled, created_at, updated_at
                 FROM content_items ORDER BY id ASC",
                (),
            )
            .await
            .map_err(|error| format!("查询内容实例失败: {error}"))?,
    };

    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取内容实例失败: {error}"))?
    {
        let mut item = map_content_item_row(&row)?;
        apply_content_item_display_text(&mut item, preferred_locale).await;
        values.push(item);
    }
    Ok(values)
}

/// 创建内容实例。
pub async fn create_content_item(
    payload: CreateContentItemRequest,
) -> Result<ContentItemView, &'static str> {
    ensure_game_exists(payload.game_id).await?;
    let content_type = find_content_type(payload.content_type_id, None)
        .await
        .ok_or("content_type_not_found")?;
    if content_type.game_id != payload.game_id {
        return Err("content_type_game_mismatch");
    }

    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "INSERT INTO content_items (game_id, content_type_id, slug, external_key, status, sort_order, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)",
        turso::params![
            payload.game_id,
            payload.content_type_id,
            normalize_optional_text(payload.slug),
            normalize_optional_text(payload.external_key),
            payload.status.as_str(),
            payload.sort_order.unwrap_or(0),
            now,
        ],
    )
    .await
    .map_err(|_| "content_item_create_failed")?;

    let id = last_insert_rowid(&conn)
        .await
        .map_err(|_| "content_item_create_failed")?;
    find_content_item(id, None)
        .await
        .ok_or("content_item_create_failed")
}

/// 更新内容实例。
pub async fn update_content_item(
    id: i64,
    payload: UpdateContentItemRequest,
) -> Result<ContentItemView, &'static str> {
    let existing = find_content_item(id, None)
        .await
        .ok_or("content_item_not_found")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "UPDATE content_items
         SET slug = ?1, external_key = ?2, status = ?3, sort_order = ?4, updated_at = ?5
         WHERE id = ?6",
        turso::params![
            normalize_optional_text(payload.slug.or(existing.slug)),
            normalize_optional_text(payload.external_key.or(existing.external_key)),
            payload.status.unwrap_or(existing.status).as_str(),
            payload.sort_order.unwrap_or(existing.sort_order),
            now,
            id,
        ],
    )
    .await
    .map_err(|_| "content_item_update_failed")?;
    find_content_item(id, None)
        .await
        .ok_or("content_item_update_failed")
}

/// 删除内容实例。
pub async fn delete_content_item(id: i64) -> Result<(), &'static str> {
    find_content_item(id, None)
        .await
        .ok_or("content_item_not_found")?;
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "DELETE FROM content_items WHERE id = ?1",
        turso::params![id],
    )
    .await
    .map_err(|_| "content_item_delete_failed")?;
    Ok(())
}

/// 查找内容实例。
pub async fn find_content_item(id: i64, locale: Option<Locale>) -> Option<ContentItemView> {
    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, game_id, content_type_id, slug, external_key, status, sort_order, enabled, created_at, updated_at
             FROM content_items WHERE id = ?1",
            turso::params![id],
        )
        .await
        .ok()?;
    let mut item = rows
        .next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_content_item_row(&row).ok())?;
    apply_content_item_display_text(&mut item, locale).await;
    Some(item)
}

/// 返回内容实例详情。
pub async fn get_content_item_detail(
    id: i64,
    locale: Option<&str>,
) -> Result<ContentItemDetailView, &'static str> {
    let preferred_locale = parse_optional_locale(locale).map_err(|_| "unsupported_locale")?;
    let item = find_content_item(id, preferred_locale)
        .await
        .ok_or("content_item_not_found")?;
    let game = find_game(item.game_id, preferred_locale)
        .await
        .ok_or("game_not_found")?;
    let content_type = find_content_type(item.content_type_id, preferred_locale)
        .await
        .ok_or("content_type_not_found")?;
    let localized_texts = list_content_item_texts(id)
        .await
        .map_err(|_| "content_item_detail_failed")?;
    let missing_locales = supported_locales()
        .into_iter()
        .filter(|locale| {
            !localized_texts
                .iter()
                .any(|text| text.locale.as_str() == locale.as_str())
        })
        .collect();
    let latest_embedding_job = latest_embedding_job_for_item(id).await;

    Ok(ContentItemDetailView {
        item,
        game,
        content_type,
        localized_texts,
        missing_locales,
        latest_embedding_job,
    })
}

/// 列出某个内容实例的全部语言文本。
pub async fn list_content_item_texts(
    content_item_id: i64,
) -> Result<Vec<ContentItemTextView>, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut rows = conn
        .query(
            "SELECT id, content_item_id, locale, name, subtitle, author, summary, body, created_at, updated_at
             FROM content_item_texts WHERE content_item_id = ?1 ORDER BY id ASC",
            turso::params![content_item_id],
        )
        .await
        .map_err(|error| format!("查询内容文本失败: {error}"))?;

    let mut values = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取内容文本失败: {error}"))?
    {
        values.push(map_content_item_text_row(&row)?);
    }
    Ok(values)
}

/// 新增或更新某个语言文本，并自动触发向量化任务。
pub async fn upsert_content_item_text(
    content_item_id: i64,
    locale: &str,
    payload: UpsertContentItemTextRequest,
    requested_by_user_id: Option<i64>,
    requested_by_label: Option<String>,
) -> Result<ContentItemTextView, &'static str> {
    let item = find_content_item(content_item_id, None)
        .await
        .ok_or("content_item_not_found")?;
    let content_type = find_content_type(item.content_type_id, None)
        .await
        .ok_or("content_type_not_found")?;
    let locale = locale.parse::<Locale>().map_err(|_| "unsupported_locale")?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    let existing = find_content_item_text(content_item_id, locale).await;

    if let Some(existing) = existing {
        conn.execute(
            "UPDATE content_item_texts
             SET name = ?1, subtitle = ?2, author = ?3, summary = ?4, body = ?5, updated_at = ?6
             WHERE id = ?7",
            turso::params![
                normalize_required_text(&payload.name, "content_item_text_name_required")?,
                normalize_optional_text(payload.subtitle),
                normalize_optional_text(payload.author),
                normalize_optional_text(payload.summary),
                normalize_optional_text(payload.body),
                now,
                existing.id,
            ],
        )
        .await
        .map_err(|_| "content_item_text_update_failed")?;
    } else {
        conn.execute(
            "INSERT INTO content_item_texts (content_item_id, locale, name, subtitle, author, summary, body, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            turso::params![
                content_item_id,
                locale.as_str(),
                normalize_required_text(&payload.name, "content_item_text_name_required")?,
                normalize_optional_text(payload.subtitle),
                normalize_optional_text(payload.author),
                normalize_optional_text(payload.summary),
                normalize_optional_text(payload.body),
                now,
            ],
        )
        .await
        .map_err(|_| "content_item_text_create_failed")?;
    }

    let saved = find_content_item_text(content_item_id, locale)
        .await
        .ok_or("content_item_text_create_failed")?;
    if content_type.supports_embedding {
        crate::vector::jobs::enqueue_for_text(
            saved.content_item_id,
            saved.id,
            "text_upsert",
            requested_by_user_id,
            requested_by_label,
        )
        .await
        .map_err(|_| "embedding_job_enqueue_failed")?;
    }
    Ok(saved)
}

/// 删除某个语言文本。
pub async fn delete_content_item_text(
    content_item_id: i64,
    locale: &str,
) -> Result<(), &'static str> {
    let locale = locale.parse::<Locale>().map_err(|_| "unsupported_locale")?;
    let text = find_content_item_text(content_item_id, locale)
        .await
        .ok_or("content_item_text_not_found")?;
    let conn = db::database()
        .connect()
        .map_err(|_| "content_db_unavailable")?;
    conn.execute(
        "DELETE FROM content_item_texts WHERE id = ?1",
        turso::params![text.id],
    )
    .await
    .map_err(|_| "content_item_text_delete_failed")?;
    Ok(())
}

/// 查找指定语言文本。
pub async fn find_content_item_text(
    content_item_id: i64,
    locale: Locale,
) -> Option<ContentItemTextView> {
    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, content_item_id, locale, name, subtitle, author, summary, body, created_at, updated_at
             FROM content_item_texts WHERE content_item_id = ?1 AND locale = ?2",
            turso::params![content_item_id, locale.as_str()],
        )
        .await
        .ok()?;
    rows.next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_content_item_text_row(&row).ok())
}

async fn find_game_text(game_id: i64, locale: Locale) -> Option<GameTextView> {
    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, game_id, locale, name, description, created_at, updated_at
             FROM game_texts WHERE game_id = ?1 AND locale = ?2",
            turso::params![game_id, locale.as_str()],
        )
        .await
        .ok()?;
    rows.next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_game_text_row(&row).ok())
}

async fn find_preferred_game_text(
    game_id: i64,
    preferred_locale: Option<Locale>,
) -> Option<GameTextView> {
    if let Some(locale) = preferred_locale
        && let Some(text) = find_game_text(game_id, locale).await
    {
        return Some(text);
    }

    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, game_id, locale, name, description, created_at, updated_at
             FROM game_texts WHERE game_id = ?1 ORDER BY id ASC LIMIT 1",
            turso::params![game_id],
        )
        .await
        .ok()?;
    rows.next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_game_text_row(&row).ok())
}

async fn find_content_type_text(
    content_type_id: i64,
    locale: Locale,
) -> Option<ContentTypeTextView> {
    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, content_type_id, locale, name, description, created_at, updated_at
             FROM content_type_texts WHERE content_type_id = ?1 AND locale = ?2",
            turso::params![content_type_id, locale.as_str()],
        )
        .await
        .ok()?;
    rows.next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_content_type_text_row(&row).ok())
}

async fn find_preferred_content_type_text(
    content_type_id: i64,
    preferred_locale: Option<Locale>,
) -> Option<ContentTypeTextView> {
    if let Some(locale) = preferred_locale
        && let Some(text) = find_content_type_text(content_type_id, locale).await
    {
        return Some(text);
    }

    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, content_type_id, locale, name, description, created_at, updated_at
             FROM content_type_texts WHERE content_type_id = ?1 ORDER BY id ASC LIMIT 1",
            turso::params![content_type_id],
        )
        .await
        .ok()?;
    rows.next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_content_type_text_row(&row).ok())
}

async fn find_preferred_content_item_text(
    content_item_id: i64,
    preferred_locale: Option<Locale>,
) -> Option<ContentItemTextView> {
    if let Some(locale) = preferred_locale
        && let Some(text) = find_content_item_text(content_item_id, locale).await
    {
        return Some(text);
    }

    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, content_item_id, locale, name, subtitle, author, summary, body, created_at, updated_at
             FROM content_item_texts WHERE content_item_id = ?1 ORDER BY id ASC LIMIT 1",
            turso::params![content_item_id],
        )
        .await
        .ok()?;
    rows.next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_content_item_text_row(&row).ok())
}

/// 返回某个内容实例最近一次向量任务。
pub async fn latest_embedding_job_for_item(content_item_id: i64) -> Option<EmbeddingJobView> {
    let conn = db::database().connect().ok()?;
    let mut rows = conn
        .query(
            "SELECT id, content_item_id, content_item_text_id, trigger_reason, status, model, error_message,
                    attempt_count, requested_by_user_id, requested_by_label, created_at, updated_at, started_at, completed_at
             FROM embedding_jobs WHERE content_item_id = ?1 ORDER BY id DESC LIMIT 1",
            turso::params![content_item_id],
        )
        .await
        .ok()?;
    rows.next()
        .await
        .ok()
        .flatten()
        .and_then(|row| map_embedding_job_row(&row).ok())
}

async fn ensure_game_exists(game_id: i64) -> Result<(), &'static str> {
    find_game(game_id, None)
        .await
        .ok_or("game_not_found")
        .map(|_| ())
}

async fn last_insert_rowid(conn: &turso::Connection) -> Result<i64, String> {
    Ok(conn.last_insert_rowid())
}

fn normalize_code(value: &str) -> Result<String, &'static str> {
    let normalized = value.trim().to_ascii_lowercase().replace(' ', "_");
    if normalized.is_empty() {
        return Err("code_required");
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

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let normalized = value.trim().to_string();
        (!normalized.is_empty()).then_some(normalized)
    })
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn map_game_row(row: &turso::Row) -> Result<GameView, String> {
    Ok(GameView {
        id: row.get(0).map_err(|error| error.to_string())?,
        code: row.get(1).map_err(|error| error.to_string())?,
        display_name: None,
        display_description: None,
        enabled: row.get::<i64>(2).map_err(|error| error.to_string())? != 0,
        created_at: row.get(3).map_err(|error| error.to_string())?,
        updated_at: row.get(4).map_err(|error| error.to_string())?,
    })
}

fn map_game_text_row(row: &turso::Row) -> Result<GameTextView, String> {
    let locale: String = row.get(2).map_err(|error| error.to_string())?;
    Ok(GameTextView {
        id: row.get(0).map_err(|error| error.to_string())?,
        game_id: row.get(1).map_err(|error| error.to_string())?,
        locale: locale
            .parse::<Locale>()
            .map_err(|error| error.to_string())?,
        name: row.get(3).map_err(|error| error.to_string())?,
        description: row.get(4).map_err(|error| error.to_string())?,
        created_at: row.get(5).map_err(|error| error.to_string())?,
        updated_at: row.get(6).map_err(|error| error.to_string())?,
    })
}

fn map_content_type_row(row: &turso::Row) -> Result<ContentTypeView, String> {
    Ok(ContentTypeView {
        id: row.get(0).map_err(|error| error.to_string())?,
        game_id: row.get(1).map_err(|error| error.to_string())?,
        code: row.get(2).map_err(|error| error.to_string())?,
        display_name: None,
        display_description: None,
        supports_i18n: row.get::<i64>(5).map_err(|error| error.to_string())? != 0,
        supports_embedding: row.get::<i64>(6).map_err(|error| error.to_string())? != 0,
        enabled: row.get::<i64>(7).map_err(|error| error.to_string())? != 0,
        created_at: row.get(8).map_err(|error| error.to_string())?,
        updated_at: row.get(9).map_err(|error| error.to_string())?,
    })
}

fn map_content_type_text_row(row: &turso::Row) -> Result<ContentTypeTextView, String> {
    let locale: String = row.get(2).map_err(|error| error.to_string())?;
    Ok(ContentTypeTextView {
        id: row.get(0).map_err(|error| error.to_string())?,
        content_type_id: row.get(1).map_err(|error| error.to_string())?,
        locale: locale
            .parse::<Locale>()
            .map_err(|error| error.to_string())?,
        name: row.get(3).map_err(|error| error.to_string())?,
        description: row.get(4).map_err(|error| error.to_string())?,
        created_at: row.get(5).map_err(|error| error.to_string())?,
        updated_at: row.get(6).map_err(|error| error.to_string())?,
    })
}

fn map_content_item_row(row: &turso::Row) -> Result<ContentItemView, String> {
    let status: String = row.get(5).map_err(|error| error.to_string())?;
    Ok(ContentItemView {
        id: row.get(0).map_err(|error| error.to_string())?,
        game_id: row.get(1).map_err(|error| error.to_string())?,
        content_type_id: row.get(2).map_err(|error| error.to_string())?,
        display_name: None,
        slug: row.get(3).map_err(|error| error.to_string())?,
        external_key: row.get(4).map_err(|error| error.to_string())?,
        status: status
            .parse::<ContentItemStatus>()
            .map_err(|error| error.to_string())?,
        sort_order: row.get(6).map_err(|error| error.to_string())?,
        enabled: row.get::<i64>(7).map_err(|error| error.to_string())? != 0,
        created_at: row.get(8).map_err(|error| error.to_string())?,
        updated_at: row.get(9).map_err(|error| error.to_string())?,
    })
}

fn map_content_item_text_row(row: &turso::Row) -> Result<ContentItemTextView, String> {
    let locale: String = row.get(2).map_err(|error| error.to_string())?;
    Ok(ContentItemTextView {
        id: row.get(0).map_err(|error| error.to_string())?,
        content_item_id: row.get(1).map_err(|error| error.to_string())?,
        locale: locale
            .parse::<Locale>()
            .map_err(|error| error.to_string())?,
        name: row.get(3).map_err(|error| error.to_string())?,
        subtitle: row.get(4).map_err(|error| error.to_string())?,
        author: row.get(5).map_err(|error| error.to_string())?,
        summary: row.get(6).map_err(|error| error.to_string())?,
        body: row.get(7).map_err(|error| error.to_string())?,
        created_at: row.get(8).map_err(|error| error.to_string())?,
        updated_at: row.get(9).map_err(|error| error.to_string())?,
    })
}

fn map_embedding_job_row(row: &turso::Row) -> Result<EmbeddingJobView, String> {
    let status: String = row.get(4).map_err(|error| error.to_string())?;
    Ok(EmbeddingJobView {
        id: row.get(0).map_err(|error| error.to_string())?,
        content_item_id: row.get(1).map_err(|error| error.to_string())?,
        content_item_text_id: row.get(2).map_err(|error| error.to_string())?,
        trigger_reason: row.get(3).map_err(|error| error.to_string())?,
        status: status
            .parse::<EmbeddingJobStatus>()
            .map_err(|error| error.to_string())?,
        model: row.get(5).map_err(|error| error.to_string())?,
        error_message: row.get(6).map_err(|error| error.to_string())?,
        attempt_count: row.get(7).map_err(|error| error.to_string())?,
        requested_by_user_id: row.get(8).map_err(|error| error.to_string())?,
        requested_by_label: row.get(9).map_err(|error| error.to_string())?,
        created_at: row.get(10).map_err(|error| error.to_string())?,
        updated_at: row.get(11).map_err(|error| error.to_string())?,
        started_at: row.get(12).map_err(|error| error.to_string())?,
        completed_at: row.get(13).map_err(|error| error.to_string())?,
    })
}

async fn apply_game_display_text(game: &mut GameView, preferred_locale: Option<Locale>) {
    let text = find_preferred_game_text(game.id, preferred_locale).await;
    if let Some(text) = text {
        game.display_name = Some(text.name);
        game.display_description = text.description;
    }
}

async fn apply_content_type_display_text(
    content_type: &mut ContentTypeView,
    preferred_locale: Option<Locale>,
) {
    let text = find_preferred_content_type_text(content_type.id, preferred_locale).await;
    if let Some(text) = text {
        content_type.display_name = Some(text.name);
        content_type.display_description = text.description;
    }
}

async fn apply_content_item_display_text(
    content_item: &mut ContentItemView,
    preferred_locale: Option<Locale>,
) {
    let text = find_preferred_content_item_text(content_item.id, preferred_locale).await;
    if let Some(text) = text {
        content_item.display_name = Some(text.name);
    }
}

fn parse_optional_locale(locale: Option<&str>) -> Result<Option<Locale>, String> {
    locale
        .map(|value| value.parse::<Locale>().map_err(|error| error.to_string()))
        .transpose()
}

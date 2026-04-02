/// 统一内容业务库建表 SQL。
pub const SCHEMA_STATEMENTS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS games (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        code TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        description TEXT,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS content_types (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        game_id INTEGER NOT NULL,
        code TEXT NOT NULL,
        name TEXT NOT NULL,
        description TEXT,
        supports_i18n INTEGER NOT NULL DEFAULT 1,
        supports_embedding INTEGER NOT NULL DEFAULT 1,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        UNIQUE(game_id, code),
        FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS game_texts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        game_id INTEGER NOT NULL,
        locale TEXT NOT NULL,
        name TEXT NOT NULL,
        description TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        UNIQUE(game_id, locale),
        FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS content_type_texts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        content_type_id INTEGER NOT NULL,
        locale TEXT NOT NULL,
        name TEXT NOT NULL,
        description TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        UNIQUE(content_type_id, locale),
        FOREIGN KEY(content_type_id) REFERENCES content_types(id) ON DELETE CASCADE
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS content_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        game_id INTEGER NOT NULL,
        content_type_id INTEGER NOT NULL,
        slug TEXT,
        external_key TEXT,
        status TEXT NOT NULL,
        sort_order INTEGER NOT NULL DEFAULT 0,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE,
        FOREIGN KEY(content_type_id) REFERENCES content_types(id) ON DELETE CASCADE
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS content_item_texts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        content_item_id INTEGER NOT NULL,
        locale TEXT NOT NULL,
        name TEXT NOT NULL,
        subtitle TEXT,
        author TEXT,
        summary TEXT,
        body TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        UNIQUE(content_item_id, locale),
        FOREIGN KEY(content_item_id) REFERENCES content_items(id) ON DELETE CASCADE
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS knowledge_documents (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        content_item_id INTEGER NOT NULL,
        content_item_text_id INTEGER NOT NULL UNIQUE,
        game_code TEXT NOT NULL,
        content_type_code TEXT NOT NULL,
        locale TEXT NOT NULL,
        title TEXT NOT NULL,
        canonical_text TEXT NOT NULL,
        model TEXT NOT NULL,
        updated_at INTEGER NOT NULL,
        FOREIGN KEY(content_item_id) REFERENCES content_items(id) ON DELETE CASCADE,
        FOREIGN KEY(content_item_text_id) REFERENCES content_item_texts(id) ON DELETE CASCADE
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS knowledge_chunks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        document_id INTEGER NOT NULL,
        content_item_id INTEGER NOT NULL,
        content_item_text_id INTEGER NOT NULL,
        game_code TEXT NOT NULL,
        content_type_code TEXT NOT NULL,
        locale TEXT NOT NULL,
        chunk_index INTEGER NOT NULL,
        text TEXT NOT NULL,
        embedding BLOB,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        UNIQUE(document_id, chunk_index),
        FOREIGN KEY(document_id) REFERENCES knowledge_documents(id) ON DELETE CASCADE,
        FOREIGN KEY(content_item_id) REFERENCES content_items(id) ON DELETE CASCADE,
        FOREIGN KEY(content_item_text_id) REFERENCES content_item_texts(id) ON DELETE CASCADE
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS embedding_jobs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        content_item_id INTEGER NOT NULL,
        content_item_text_id INTEGER NOT NULL,
        trigger_reason TEXT NOT NULL,
        status TEXT NOT NULL,
        model TEXT NOT NULL,
        error_message TEXT,
        attempt_count INTEGER NOT NULL DEFAULT 0,
        requested_by_user_id INTEGER,
        requested_by_label TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        started_at INTEGER,
        completed_at INTEGER,
        FOREIGN KEY(content_item_id) REFERENCES content_items(id) ON DELETE CASCADE,
        FOREIGN KEY(content_item_text_id) REFERENCES content_item_texts(id) ON DELETE CASCADE
    )
    "#,
    "CREATE INDEX IF NOT EXISTS idx_content_types_game_id ON content_types(game_id)",
    "CREATE INDEX IF NOT EXISTS idx_game_texts_game_id ON game_texts(game_id)",
    "CREATE INDEX IF NOT EXISTS idx_content_type_texts_content_type_id ON content_type_texts(content_type_id)",
    "CREATE INDEX IF NOT EXISTS idx_content_items_game_type ON content_items(game_id, content_type_id)",
    "CREATE INDEX IF NOT EXISTS idx_content_item_texts_item_id ON content_item_texts(content_item_id)",
    "CREATE INDEX IF NOT EXISTS idx_embedding_jobs_status_created_at ON embedding_jobs(status, created_at)",
    "CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_lookup ON knowledge_chunks(game_code, content_type_code, locale)",
];

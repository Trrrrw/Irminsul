pub const SCHEMA_STATEMENTS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS ADMIN_USERS (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL UNIQUE,
        email TEXT UNIQUE,
        password_hash TEXT NOT NULL,
        role TEXT NOT NULL,
        status TEXT NOT NULL,
        must_change_password INTEGER NOT NULL,
        must_change_username INTEGER NOT NULL,
        must_set_email INTEGER NOT NULL,
        last_login_at INTEGER,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS ADMIN_SESSIONS (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        admin_user_id INTEGER NOT NULL,
        token_hash TEXT NOT NULL UNIQUE,
        csrf_token_hash TEXT NOT NULL UNIQUE,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        expires_at INTEGER NOT NULL,
        last_seen_at INTEGER NOT NULL,
        revoked_at INTEGER,
        created_ip TEXT,
        last_seen_ip TEXT,
        user_agent TEXT
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS ADMIN_INVITATIONS (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        token_hash TEXT NOT NULL UNIQUE,
        role TEXT NOT NULL,
        status TEXT NOT NULL,
        invited_email TEXT,
        note TEXT,
        created_by_user_id INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        expires_at INTEGER NOT NULL,
        consumed_at INTEGER,
        consumed_by_user_id INTEGER
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS EMBEDDING_PROVIDERS (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        code TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        base_url TEXT NOT NULL,
        embeddings_path TEXT NOT NULL,
        enabled INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS EMBEDDING_API_KEYS (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        provider_id INTEGER NOT NULL,
        name TEXT NOT NULL,
        api_key TEXT NOT NULL,
        enabled INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS EMBEDDING_SETTINGS (
        id INTEGER PRIMARY KEY,
        default_provider_id INTEGER,
        current_model TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS AUDIT_LOGS (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        actor_type TEXT NOT NULL,
        actor_user_id INTEGER,
        actor_label TEXT,
        action TEXT NOT NULL,
        target_type TEXT NOT NULL,
        target_id TEXT,
        summary TEXT NOT NULL,
        metadata_json TEXT,
        ip TEXT,
        user_agent TEXT,
        created_at INTEGER NOT NULL
    )
    "#,
];

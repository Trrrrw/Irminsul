use std::path::Path;
use std::sync::OnceLock;

use turso::{Builder, Database};

mod schema;

/// 管理后台数据库实例。
static ADMIN_DB: OnceLock<Database> = OnceLock::new();

/// 初始化后台数据库，并同步后台所需的全部表结构。
pub async fn init<P: AsRef<Path>>(path: P) {
    let db = Builder::new_local(path.as_ref().to_string_lossy().as_ref())
        .build()
        .await
        .expect("admin db should connect");
    let conn = db.connect().expect("admin db connection should open");
    conn.execute("PRAGMA foreign_keys = ON", ())
        .await
        .expect("admin db should enable foreign keys");

    for statement in schema::SCHEMA_STATEMENTS {
        conn.execute(statement, ())
            .await
            .unwrap_or_else(|error| panic!("admin db schema should sync: {error}"));
    }

    ADMIN_DB.set(db).expect("admin db should be set");
    crate::admin::services::bootstrap::ensure_initial_owner().await;
}

/// 返回后台数据库实例。
pub fn database() -> &'static Database {
    ADMIN_DB.get().expect("admin db should be set")
}

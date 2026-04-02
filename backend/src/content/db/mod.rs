use std::path::Path;
use std::sync::OnceLock;

use turso::{Builder, Database};

mod schema;

/// 统一内容业务库连接池。
static CONTENT_DB: OnceLock<Database> = OnceLock::new();

/// 初始化统一内容库。
pub async fn init<P: AsRef<Path>>(path: P) {
    let db = Builder::new_local(path.as_ref().to_string_lossy().as_ref())
        .build()
        .await
        .expect("content db should connect");
    let conn = db.connect().expect("content db connection should open");
    conn.execute("PRAGMA foreign_keys = ON", ())
        .await
        .expect("content db should enable foreign keys");

    for statement in schema::SCHEMA_STATEMENTS {
        conn.execute(statement, ())
            .await
            .unwrap_or_else(|error| panic!("content db schema should sync: {error}"));
    }

    CONTENT_DB.set(db).expect("content db should be set");
}

/// 返回统一内容库实例。
pub fn database() -> &'static Database {
    CONTENT_DB.get().expect("content db should be set")
}

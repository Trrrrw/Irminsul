use std::path::Path;
use std::sync::OnceLock;

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

mod schema;

/// 管理后台数据库连接。
static ADMIN_DB: OnceLock<DatabaseConnection> = OnceLock::new();

/// 初始化后台数据库，并同步后台所需的全部表结构。
pub async fn init<P: AsRef<Path>>(path: P) {
    let url = format!(
        "sqlite://{}?mode=rwc",
        path.as_ref().to_string_lossy().replace('\\', "/")
    );
    let db = Database::connect(&url)
        .await
        .expect("admin db should connect");

    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA foreign_keys = ON".to_string(),
    ))
    .await
    .expect("admin db should enable foreign keys");

    for statement in schema::SCHEMA_STATEMENTS {
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            (*statement).to_string(),
        ))
        .await
        .unwrap_or_else(|error| panic!("admin db schema should sync: {error}"));
    }

    ADMIN_DB.set(db).expect("admin db should be set");
    crate::admin::services::bootstrap::ensure_initial_owner().await;
}

/// 返回后台数据库连接。
pub fn database() -> &'static DatabaseConnection {
    ADMIN_DB.get().expect("admin db should be set")
}

use sea_orm::entity::prelude::DatabaseConnection;
use sea_orm::{ConnectOptions, Database};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use crate::{admin, audit, entities};

pub static SEAORM_POOL: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).expect("failed to create database directory");
        }
    }

    let url = format!("sqlite://{}?mode=rwc", path.to_str().unwrap());
    let mut opt = ConnectOptions::new(url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(false);

    let pool = Database::connect(opt)
        .await
        .expect("db connection should connect");

    let builder = entities::register_all(pool.get_schema_builder());
    let builder = admin::entities::register_all(builder);
    let builder = audit::entities::register_all(builder);
    builder
        .sync(&pool)
        .await
        .expect("seaorm pool should be set");

    SEAORM_POOL.set(pool).expect("seaorm pool should be set");
    admin::services::bootstrap::ensure_initial_owner().await;
}

pub fn pool() -> &'static DatabaseConnection {
    SEAORM_POOL.get().expect("seaorm pool should set")
}

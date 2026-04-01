use sea_orm::entity::prelude::DatabaseConnection;
use sea_orm::{ConnectOptions, Database};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use crate::admin;

pub static ADMIN_POOL: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init<P: AsRef<Path>>(path: P) {
    let url = format!("sqlite://{}?mode=rwc", path.as_ref().to_str().unwrap());
    let mut opt = ConnectOptions::new(url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(false);

    let pool = Database::connect(opt)
        .await
        .expect("admin db connection should connect");

    let builder = admin::entities::register_all(pool.get_schema_builder());
    builder.sync(&pool).await.expect("admin pool should be set");

    ADMIN_POOL.set(pool).expect("admin pool should be set");
    admin::services::bootstrap::ensure_initial_owner().await;
}

pub fn pool() -> &'static DatabaseConnection {
    ADMIN_POOL.get().expect("admin pool should set")
}

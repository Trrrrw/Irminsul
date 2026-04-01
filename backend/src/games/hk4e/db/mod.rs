use sea_orm::entity::prelude::DatabaseConnection;
use sea_orm::{ConnectOptions, Database};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

pub mod entities;

pub static HK4E_POOL: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init<P: AsRef<Path>>(path: P) {
    let url = format!("sqlite://{}?mode=rwc", path.as_ref().to_string_lossy());
    let mut opt = ConnectOptions::new(url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(false);

    let pool = Database::connect(opt)
        .await
        .expect("hk4e db connection should connect");

    let builder = entities::register_all(pool.get_schema_builder());
    builder.sync(&pool).await.expect("hk4e db should sync");

    HK4E_POOL.set(pool).expect("hk4e pool should be set");
}

pub fn pool() -> &'static DatabaseConnection {
    HK4E_POOL.get().expect("hk4e pool should set")
}

use sea_orm::entity::prelude::DatabaseConnection;
use sea_orm::{ConnectOptions, Database};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use crate::entities::*;

pub static SEAORM_POOL: OnceLock<DatabaseConnection> = OnceLock::new();

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
        .expect("db connection should connect");
    pool.get_schema_builder()
        .register(users::Entity)
        .register(games::Entity)
        .register(games::i18n::Entity)
        .sync(&pool)
        .await
        .expect("sync all schema should success");
    SEAORM_POOL.set(pool).expect("seaorm pool should be set");
}

pub fn pool() -> &'static DatabaseConnection {
    SEAORM_POOL.get().expect("seaorm pool should set")
}

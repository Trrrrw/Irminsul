use sea_orm::entity::prelude::DatabaseConnection;
use std::fs;
use std::path::Path;

use crate::{admin, games};

pub async fn init<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    fs::create_dir_all(path).expect("failed to create database directory");

    admin::db::init(path.join("admin.db")).await;
    games::hk4e::db::init(path.join("hk4e.db")).await;
}

pub fn pool(db_name: &str) -> &'static DatabaseConnection {
    match db_name {
        "admin" => admin::db::pool(),
        "hk4e" => games::hk4e::db::pool(),
        other => panic!("unknown database pool: {other}"),
    }
}

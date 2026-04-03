use std::fs;
use std::path::Path;

use crate::{admin, config, mongo};

pub async fn init<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    fs::create_dir_all(path).expect("failed to create database directory");

    config::init(path).await;
    admin::db::init(path.join("admin.db")).await;
    mongo::init().await;
}

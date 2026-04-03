use std::sync::OnceLock;

use mongodb::{Client, Database, IndexModel, bson::doc, options::IndexOptions};

static MONGO_DATABASE: OnceLock<Database> = OnceLock::new();

pub async fn init() {
    let uri = std::env::var("IRMINSUL_MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://admin:123456@127.0.0.1:27017/?authSource=admin".to_string());
    let database_name =
        std::env::var("IRMINSUL_MONGODB_DATABASE").unwrap_or_else(|_| "irminsul".to_string());

    let client = Client::with_uri_str(&uri)
        .await
        .expect("mongodb client should connect");
    let database = client.database(&database_name);

    ensure_indexes(&database).await;

    MONGO_DATABASE
        .set(database)
        .expect("mongo database should be initialized once");
}

pub fn database() -> &'static Database {
    MONGO_DATABASE
        .get()
        .expect("mongo database should be initialized")
}

async fn ensure_indexes(database: &Database) {
    let schema_collection = database.collection::<mongodb::bson::Document>("_schemas");
    let schema_key_index = IndexModel::builder()
        .keys(doc! { "key": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = schema_collection.create_index(schema_key_index).await;
}

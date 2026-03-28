use sea_orm::SchemaBuilder;

pub mod logs;

pub fn register_all(builder: SchemaBuilder) -> SchemaBuilder {
    builder.register(logs::Entity)
}

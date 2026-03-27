use sea_orm::SchemaBuilder;

pub mod characters;
pub mod games;
pub mod users;

pub fn register_all(builder: SchemaBuilder) -> SchemaBuilder {
    builder
        .register(users::Entity)
        .register(games::Entity)
        .register(games::i18n::Entity)
        .register(games::alias::Entity)
        .register(characters::Entity)
        .register(characters::i18n::Entity)
        .register(characters::alias::Entity)
}

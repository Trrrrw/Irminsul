use sea_orm::SchemaBuilder;

pub mod characters;
pub mod elements;
pub mod game_info;
pub mod weapons;

pub fn register_all(builder: SchemaBuilder) -> SchemaBuilder {
    builder
        .register(game_info::Entity)
        .register(characters::Entity)
        .register(elements::elements::Entity)
        .register(elements::elemental_reactions::Entity)
        .register(elements::elemental_reaction_elements::Entity)
        .register(weapons::weapon_types::Entity)
        .register(weapons::weapons::Entity)
}

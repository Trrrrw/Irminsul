use sea_orm::SchemaBuilder;

pub mod audit_logs;
pub mod invitations;
pub mod sessions;
pub mod users;

pub fn register_all(builder: SchemaBuilder) -> SchemaBuilder {
    builder
        .register(users::Entity)
        .register(sessions::Entity)
        .register(invitations::Entity)
        .register(audit_logs::Entity)
}

pub mod alias;
pub mod i18n;

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "CHARACTERS")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,

    #[sea_orm(has_many)]
    pub i18ns: HasMany<i18n::Entity>,
    #[sea_orm(has_many)]
    pub aliases: HasMany<alias::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

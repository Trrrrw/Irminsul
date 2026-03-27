use sea_orm::entity::prelude::*;

use crate::{entities::characters, models::lang::Lang};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "CHARACTERS_ALIAS")]
pub struct Model {
    pub character_id: i32,
    #[sea_orm(primary_key)]
    pub lang: Lang,
    #[sea_orm(primary_key)]
    pub alias: String,

    #[sea_orm(belongs_to, from = "character_id", to = "id")]
    pub character: HasOne<characters::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

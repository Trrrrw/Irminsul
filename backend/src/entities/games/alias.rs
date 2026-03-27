use sea_orm::entity::prelude::*;

use crate::{entities::games, models::lang::Lang};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "GAMES_ALIAS")]
pub struct Model {
    pub game_id: i32,
    #[sea_orm(primary_key)]
    pub lang: Lang,
    #[sea_orm(primary_key)]
    pub alias: String,

    #[sea_orm(belongs_to, from = "game_id", to = "id")]
    pub game: HasOne<games::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

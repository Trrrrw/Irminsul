use sea_orm::entity::prelude::*;

use crate::{entities::games, models::lang::Lang};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "GAMES_I18N")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub game_id: i32,
    #[sea_orm(primary_key)]
    pub lang: Lang,
    pub translation: String,

    #[sea_orm(belongs_to, from = "game_id", to = "id")]
    pub game: HasOne<games::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

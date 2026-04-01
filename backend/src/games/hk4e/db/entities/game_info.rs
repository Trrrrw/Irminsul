use sea_orm::entity::prelude::*;

/// 游戏基础信息表。
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "GAME_INFO")]
pub struct Model {
    // === 自增主键 ===
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    #[sea_orm(unique)]
    pub code_name: String,

    // === 展示信息 ===
    pub name: String,
}

impl ActiveModelBehavior for ActiveModel {}

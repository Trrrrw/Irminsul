use sea_orm::entity::prelude::*;

/// 元素实体表。
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ELEMENTS")]
pub struct Model {
    // === 自增主键 ===
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    // === 关联关系 ===
    #[sea_orm(has_many)]
    pub characters: HasMany<super::super::characters::Entity>,
    #[sea_orm(has_many)]
    pub elemental_reaction_elements: HasMany<super::elemental_reaction_elements::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

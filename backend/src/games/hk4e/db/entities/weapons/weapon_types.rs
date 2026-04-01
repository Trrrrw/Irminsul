use sea_orm::entity::prelude::*;

/// 武器类型实体表。
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "WEAPON_TYPES")]
pub struct Model {
    // === 自增主键 ===
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    // === 关联关系 ===
    #[sea_orm(has_many)]
    pub characters: HasMany<super::super::characters::Entity>,
    #[sea_orm(has_many)]
    pub weapons: HasMany<super::weapons::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

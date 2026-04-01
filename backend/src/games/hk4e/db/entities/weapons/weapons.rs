use sea_orm::entity::prelude::*;

/// 武器实体表。
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "WEAPONS")]
pub struct Model {
    // === 自增主键 ===
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    // === 关联字段 ===
    pub weapon_type_id: i64,

    // === 关联关系 ===
    #[sea_orm(belongs_to, from = "weapon_type_id", to = "id")]
    pub weapon_type: HasOne<super::weapon_types::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

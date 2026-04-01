use sea_orm::entity::prelude::*;

/// 角色实体表。
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "CHARACTERS")]
pub struct Model {
    // === 自增主键 ===
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,

    // === 基础信息 ===
    #[sea_orm(unique)]
    pub name: String,
    pub quality: i32,

    // === 关联字段 ===
    pub element_id: i64,
    pub weapon_type_id: i64,

    // === 关联关系 ===
    #[sea_orm(belongs_to, from = "element_id", to = "id")]
    pub element: HasOne<super::elements::elements::Entity>,
    #[sea_orm(belongs_to, from = "weapon_type_id", to = "id")]
    pub weapon_type: HasOne<super::weapons::weapon_types::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

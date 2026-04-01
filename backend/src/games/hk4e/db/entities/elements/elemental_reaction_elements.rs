use sea_orm::entity::prelude::*;

/// 元素反应与元素的关联表。
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "ELEMENTAL_REACTION_ELEMENTS")]
pub struct Model {
    // === 复合主键 ===
    #[sea_orm(primary_key, auto_increment = false)]
    pub reaction_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub element_id: i64,

    // === 关联关系 ===
    #[sea_orm(belongs_to, from = "reaction_id", to = "id")]
    pub reaction: Option<super::elemental_reactions::Entity>,
    #[sea_orm(belongs_to, from = "element_id", to = "id")]
    pub element: Option<super::elements::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

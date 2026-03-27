use sea_orm::entity::prelude::*;

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Eq)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Lang {
    #[sea_orm(string_value = "zh-CN")]
    ZhCN,
    #[sea_orm(string_value = "zh-TW")]
    ZhTW,
    #[sea_orm(string_value = "en-US")]
    EnUS,
    #[sea_orm(string_value = "ja-JP")]
    JaJP,
}

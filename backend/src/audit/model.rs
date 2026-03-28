use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 审计日志中的操作者类型。
#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum AuditActorType {
    #[sea_orm(string_value = "admin_user")]
    AdminUser,
    #[sea_orm(string_value = "plugin")]
    Plugin,
    #[sea_orm(string_value = "system")]
    System,
    #[sea_orm(string_value = "scheduler")]
    Scheduler,
}

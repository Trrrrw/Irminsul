use crate::admin::model::{AdminRole, AdminUserStatus};

/// 管理后台用户记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    // === 自增主键 ===
    pub id: i64,

    // === 账号基础信息 ===
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,

    // === 权限与状态 ===
    pub role: AdminRole,
    pub status: AdminUserStatus,

    // === 首次登录约束 ===
    pub must_change_password: bool,
    pub must_change_username: bool,
    pub must_set_email: bool,

    // === 审计时间 ===
    pub last_login_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

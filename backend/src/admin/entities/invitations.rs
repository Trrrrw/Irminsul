use crate::admin::model::{AdminRole, InvitationStatus};

/// 管理后台邀请码记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    // === 自增主键 ===
    pub id: i64,

    // === 邀请码主体 ===
    pub token_hash: String,
    pub role: AdminRole,
    pub status: InvitationStatus,
    pub invited_email: Option<String>,
    pub note: Option<String>,

    // === 审计信息 ===
    pub created_by_user_id: i64,
    pub created_at: i64,
    pub expires_at: i64,
    pub consumed_at: Option<i64>,
    pub consumed_by_user_id: Option<i64>,
}

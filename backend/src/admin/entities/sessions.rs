/// 管理后台会话记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    // === 自增主键 ===
    pub id: i64,

    // === 会话关联 ===
    pub admin_user_id: i64,
    pub token_hash: String,
    pub csrf_token_hash: String,

    // === 生命周期 ===
    pub created_at: i64,
    pub updated_at: i64,
    pub expires_at: i64,
    pub last_seen_at: i64,
    pub revoked_at: Option<i64>,

    // === 访问上下文 ===
    pub created_ip: Option<String>,
    pub last_seen_ip: Option<String>,
    pub user_agent: Option<String>,
}

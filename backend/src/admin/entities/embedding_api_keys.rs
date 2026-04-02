/// Embedding API key 记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    // === 自增主键 ===
    pub id: i64,

    // === 关联与密钥 ===
    pub provider_id: i64,
    pub name: String,
    pub api_key: String,
    pub enabled: bool,

    // === 审计时间 ===
    pub created_at: i64,
    pub updated_at: i64,
}

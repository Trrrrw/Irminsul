/// Embedding 提供方记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    // === 自增主键 ===
    pub id: i64,

    // === 提供方信息 ===
    pub code: String,
    pub name: String,
    pub base_url: String,
    pub embeddings_path: String,
    pub enabled: bool,

    // === 审计时间 ===
    pub created_at: i64,
    pub updated_at: i64,
}

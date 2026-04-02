/// 全局 embedding 设置记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    // === 固定主键 ===
    pub id: i64,

    // === 当前配置 ===
    pub default_provider_id: Option<i64>,
    pub current_model: String,

    // === 审计时间 ===
    pub updated_at: i64,
}

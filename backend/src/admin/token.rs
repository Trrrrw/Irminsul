use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

/// 生成随机十六进制令牌。
pub fn generate_token(byte_len: usize) -> String {
    let mut bytes = vec![0_u8; byte_len];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// 使用 SHA-256 对令牌做不可逆摘要，避免数据库泄露时直接暴露会话原文。
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

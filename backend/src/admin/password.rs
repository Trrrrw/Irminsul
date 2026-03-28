use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use rand::{RngCore, seq::SliceRandom};

/// 校验密码策略。
pub fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("password must be at least 8 characters long");
    }
    if password.chars().any(char::is_whitespace) {
        return Err("password cannot contain whitespace");
    }
    if !password.chars().any(|value| value.is_ascii_lowercase()) {
        return Err("password must contain a lowercase letter");
    }
    if !password.chars().any(|value| value.is_ascii_uppercase()) {
        return Err("password must contain an uppercase letter");
    }
    if !password.chars().any(|value| value.is_ascii_digit()) {
        return Err("password must contain a digit");
    }
    if !password.chars().any(|value| !value.is_ascii_alphanumeric()) {
        return Err("password must contain a symbol");
    }
    Ok(())
}

/// 对密码执行 Argon2id 哈希。
pub fn hash_password(password: &str) -> Result<String, &'static str> {
    validate_password(password)?;
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "failed to hash password")
}

/// 验证密码是否匹配指定哈希。
pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// 生成满足当前密码策略的临时密码。
pub fn generate_temporary_password() -> String {
    let mut chars = vec!['a', 'A', '1', '!'];
    let charset = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%^&*()-_=+";
    let mut random = [0_u8; 8];
    OsRng.fill_bytes(&mut random);
    chars.extend(
        random
            .into_iter()
            .map(|value| charset[(value as usize) % charset.len()] as char),
    );
    chars.shuffle(&mut OsRng);
    chars.into_iter().collect()
}

/// 生成用于初始化账号的随机用户名。
pub fn generate_temporary_username(prefix: &str) -> String {
    let suffix = crate::admin::token::generate_token(4);
    format!("{prefix}-{}", &suffix[..8])
}

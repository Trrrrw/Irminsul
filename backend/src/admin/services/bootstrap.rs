use std::str::FromStr;

use crate::admin::{
    db,
    model::{AdminRole, AdminUserStatus},
    password::{generate_temporary_password, generate_temporary_username, hash_password},
    services::embedding,
};

/// 确保系统至少存在一个 owner 账号。
pub async fn ensure_initial_owner() {
    let _ = embedding::ensure_embedding_settings().await;
    let Ok(conn) = db::database().connect() else {
        return;
    };

    if has_owner(&conn).await {
        return;
    }

    let env_username = std::env::var("IRMINSUL_ADMIN_OWNER_USERNAME").ok();
    let env_password = std::env::var("IRMINSUL_ADMIN_OWNER_PASSWORD").ok();
    let username = env_username
        .clone()
        .unwrap_or_else(|| generate_temporary_username("owner"));
    let password = match env_password.clone() {
        Some(password) => {
            crate::admin::password::validate_password(&password)
                .expect("IRMINSUL_ADMIN_OWNER_PASSWORD does not satisfy password policy");
            password
        }
        None => generate_temporary_password(),
    };
    let password_hash = hash_password(&password).expect("initial owner password should hash");
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let must_change_username = env_username.is_none();
    let must_change_password = env_password.is_none();

    conn.execute(
        "INSERT INTO ADMIN_USERS
         (username, email, password_hash, role, status, must_change_password, must_change_username, must_set_email, last_login_at, created_at, updated_at)
         VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, 1, NULL, ?7, ?7)",
        turso::params![
            username.clone(),
            password_hash,
            AdminRole::Owner.as_str(),
            AdminUserStatus::Active.as_str(),
            if must_change_password { 1 } else { 0 },
            if must_change_username { 1 } else { 0 },
            now,
        ],
    )
    .await
    .expect("failed to create initial owner");

    println!("Initial owner account created.");
    println!("Username: {username}");
    println!("Password: {password}");
    if must_change_username || must_change_password {
        println!("The first login must complete account initialization:");
        if must_change_username {
            println!("- Change the username");
        }
        if must_change_password {
            println!("- Change the password");
        }
        println!("- Set an email address");
    } else {
        println!("The first login must set an email address.");
    }
}

async fn has_owner(conn: &turso::Connection) -> bool {
    let Ok(mut rows) = conn
        .query(
            "SELECT role FROM ADMIN_USERS WHERE role = ?1 LIMIT 1",
            turso::params![AdminRole::Owner.as_str()],
        )
        .await
    else {
        return false;
    };

    match rows.next().await {
        Ok(Some(row)) => row
            .get::<String>(0)
            .ok()
            .and_then(|role| AdminRole::from_str(&role).ok())
            .is_some_and(|role| role == AdminRole::Owner),
        _ => false,
    }
}

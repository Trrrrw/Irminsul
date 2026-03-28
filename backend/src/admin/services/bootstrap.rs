use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    admin::{
        entities::users,
        model::{AdminRole, AdminUserStatus},
        password::{generate_temporary_password, generate_temporary_username, hash_password},
    },
    db,
};

/// 确保系统至少存在一个 owner 账号。
pub async fn ensure_initial_owner() {
    let pool = db::pool();
    let has_owner = users::Entity::find()
        .filter(users::Column::Role.eq(AdminRole::Owner))
        .one(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if has_owner {
        return;
    }

    let username = std::env::var("IRMINSUL_ADMIN_OWNER_USERNAME")
        .unwrap_or_else(|_| generate_temporary_username("owner"));
    let password = match std::env::var("IRMINSUL_ADMIN_OWNER_PASSWORD") {
        Ok(password) => {
            crate::admin::password::validate_password(&password)
                .expect("IRMINSUL_ADMIN_OWNER_PASSWORD does not satisfy password policy");
            password
        }
        Err(_) => generate_temporary_password(),
    };
    let password_hash = hash_password(&password).expect("initial owner password should hash");
    let now = crate::admin::middlewares::auth::unix_timestamp();

    let owner = users::ActiveModel {
        username: Set(username.clone()),
        email: Set(None),
        password_hash: Set(password_hash),
        role: Set(AdminRole::Owner),
        status: Set(AdminUserStatus::Active),
        must_change_password: Set(false),
        must_change_username: Set(false),
        must_set_email: Set(true),
        last_login_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let _ = owner
        .insert(pool)
        .await
        .expect("failed to create initial owner");

    println!("Initial owner account created.");
    println!("Username: {username}");
    println!("Password: {password}");
    println!("The first login must set an email address.");
}

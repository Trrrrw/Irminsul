use std::str::FromStr;

use crate::admin::{
    db,
    dto::users::{AdminUserView, UpdateSelfProfileRequest},
    entities::users,
    model::{AdminRole, AdminUserStatus},
    password::{hash_password, verify_password},
    services::audit::write_audit_log,
};

/// 列出管理员。
pub async fn list_users() -> Vec<AdminUserView> {
    let Ok(conn) = db::database().connect() else {
        return Vec::new();
    };
    let Ok(mut rows) = conn
        .query(
            "SELECT id, username, email, password_hash, role, status, must_change_password, must_change_username, must_set_email, last_login_at, created_at, updated_at
             FROM ADMIN_USERS ORDER BY id ASC",
            (),
        )
        .await
    else {
        return Vec::new();
    };

    let mut values = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        if let Ok(user) = map_user_row(&row) {
            values.push(AdminUserView::from(user));
        }
    }
    values
}

/// 启用或禁用管理员。
pub async fn set_user_enabled(
    target_user_id: i64,
    enabled: bool,
    actor_user_id: i64,
    actor_role: &AdminRole,
    ip: Option<String>,
    user_agent: Option<String>,
) -> Result<users::Model, &'static str> {
    let conn = db::database().connect().map_err(|_| "db_unavailable")?;
    let user = find_user_by_id(&conn, target_user_id)
        .await
        .ok_or("user_not_found")?;

    if !enabled && actor_user_id == target_user_id {
        return Err("cannot_disable_self");
    }
    if user.role == AdminRole::Owner && actor_role != &AdminRole::Owner {
        return Err("owner_management_forbidden");
    }
    if !enabled && user.role == AdminRole::Owner && owner_count(&conn).await <= 1 {
        return Err("cannot_disable_last_owner");
    }

    let next_status = if enabled {
        AdminUserStatus::Active
    } else {
        AdminUserStatus::Disabled
    };
    let now = crate::admin::middlewares::auth::unix_timestamp();
    conn.execute(
        "UPDATE ADMIN_USERS SET status = ?1, updated_at = ?2 WHERE id = ?3",
        turso::params![next_status.as_str(), now, target_user_id],
    )
    .await
    .map_err(|_| "user_status_update_failed")?;

    let user = find_user_by_id(&conn, target_user_id)
        .await
        .ok_or("user_status_update_failed")?;

    write_audit_log(
        Some(actor_user_id),
        None,
        if enabled {
            "enable_user"
        } else {
            "disable_user"
        },
        "admin_user",
        Some(user.id.to_string()),
        if enabled {
            "enabled admin user"
        } else {
            "disabled admin user"
        },
        Some(serde_json::json!({
            "enabled": enabled,
            "role": user.role.as_str(),
        })),
        ip,
        user_agent,
    )
    .await;

    Ok(user)
}

/// 更新当前管理员自己的资料。
pub async fn update_self_profile(
    current_user_id: i64,
    payload: UpdateSelfProfileRequest,
) -> Result<users::Model, &'static str> {
    let conn = db::database().connect().map_err(|_| "db_unavailable")?;
    let user = find_user_by_id(&conn, current_user_id)
        .await
        .ok_or("user_not_found")?;

    let new_username = payload
        .username
        .as_deref()
        .map(normalize_username)
        .unwrap_or_else(|| user.username.clone());
    let new_email = payload
        .email
        .as_deref()
        .map(normalize_optional_email)
        .unwrap_or_else(|| user.email.clone());

    if new_username.is_empty() {
        return Err("username_required");
    }
    if user.must_change_username && new_username == user.username {
        return Err("username_change_required");
    }
    if user.must_set_email && new_email.is_none() {
        return Err("email_required");
    }
    if user.role == AdminRole::Owner && new_email.is_none() {
        return Err("owner_email_required");
    }

    let (password_hash, must_change_password) =
        if let Some(new_password) = payload.new_password.as_deref() {
            if !user.must_change_password {
                let Some(current_password) = payload.current_password.as_deref() else {
                    return Err("current_password_required");
                };
                if !verify_password(current_password, &user.password_hash) {
                    return Err("current_password_invalid");
                }
            }
            (
                hash_password(new_password).map_err(|_| "password_policy_failed")?,
                false,
            )
        } else if user.must_change_password {
            return Err("password_change_required");
        } else {
            (user.password_hash.clone(), false)
        };

    let now = crate::admin::middlewares::auth::unix_timestamp();
    conn.execute(
        "UPDATE ADMIN_USERS
         SET username = ?1, email = ?2, password_hash = ?3, must_change_password = ?4, must_change_username = ?5, must_set_email = ?6, updated_at = ?7
         WHERE id = ?8",
        turso::params![
            new_username,
            new_email.clone(),
            password_hash,
            if must_change_password { 1 } else { 0 },
            0,
            if new_email.is_some() { 0 } else { user.must_set_email as i64 },
            now,
            current_user_id,
        ],
    )
    .await
    .map_err(|_| "profile_update_failed")?;

    find_user_by_id(&conn, current_user_id)
        .await
        .ok_or("profile_update_failed")
}

fn normalize_username(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_optional_email(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

async fn owner_count(conn: &turso::Connection) -> i64 {
    let Ok(mut rows) = conn
        .query(
            "SELECT COUNT(*) FROM ADMIN_USERS WHERE role = ?1 AND status = ?2",
            turso::params![AdminRole::Owner.as_str(), AdminUserStatus::Active.as_str()],
        )
        .await
    else {
        return 0;
    };

    match rows.next().await {
        Ok(Some(row)) => row.get(0).unwrap_or(0),
        _ => 0,
    }
}

async fn find_user_by_id(conn: &turso::Connection, user_id: i64) -> Option<users::Model> {
    let mut rows = conn
        .query(
            "SELECT id, username, email, password_hash, role, status, must_change_password, must_change_username, must_set_email, last_login_at, created_at, updated_at
             FROM ADMIN_USERS WHERE id = ?1 LIMIT 1",
            turso::params![user_id],
        )
        .await
        .ok()?;
    let row = rows.next().await.ok().flatten()?;
    map_user_row(&row).ok()
}

fn map_user_row(row: &turso::Row) -> Result<users::Model, String> {
    Ok(users::Model {
        id: row.get(0).map_err(|error| error.to_string())?,
        username: row.get(1).map_err(|error| error.to_string())?,
        email: row.get(2).map_err(|error| error.to_string())?,
        password_hash: row.get(3).map_err(|error| error.to_string())?,
        role: AdminRole::from_str(&row.get::<String>(4).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?,
        status: AdminUserStatus::from_str(
            &row.get::<String>(5).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?,
        must_change_password: row.get::<i64>(6).map_err(|error| error.to_string())? != 0,
        must_change_username: row.get::<i64>(7).map_err(|error| error.to_string())? != 0,
        must_set_email: row.get::<i64>(8).map_err(|error| error.to_string())? != 0,
        last_login_at: row.get(9).map_err(|error| error.to_string())?,
        created_at: row.get(10).map_err(|error| error.to_string())?,
        updated_at: row.get(11).map_err(|error| error.to_string())?,
    })
}

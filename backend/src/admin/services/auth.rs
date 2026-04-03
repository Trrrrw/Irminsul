use salvo::Request;
use sea_orm::Value;

use crate::admin::{
    db,
    entities::{invitations, sessions, users},
    middlewares::auth::unix_timestamp,
    model::{AdminUserStatus, InvitationStatus},
    password::{hash_password, verify_password},
    repository,
    services::{audit::write_audit_log, rate_limit, users::find_user_by_id},
    token::{generate_token, hash_token},
};

pub const SESSION_TTL_SECONDS: i64 = 60 * 60 * 24 * 14;

/// 管理员登录，并在成功后创建会话。
pub async fn login(
    identifier: &str,
    password: &str,
    req: &Request,
) -> Result<(users::Model, String, String), &'static str> {
    let normalized_identifier = identifier.trim().to_ascii_lowercase();
    let ip = client_ip(req);
    let now = unix_timestamp();
    if rate_limit::check_login_allowed(&normalized_identifier, ip.as_deref(), now).is_some() {
        return Err("too_many_attempts");
    }

    let Some(user) = find_user_by_identifier(db::database(), &normalized_identifier).await else {
        rate_limit::record_login_failure(&normalized_identifier, ip.as_deref(), now);
        return Err("invalid_credentials");
    };

    if user.status != AdminUserStatus::Active {
        return Err("account_disabled");
    }
    if !verify_password(password, &user.password_hash) {
        rate_limit::record_login_failure(&normalized_identifier, ip.as_deref(), now);
        return Err("invalid_credentials");
    }

    rate_limit::clear_login_failures(&normalized_identifier, ip.as_deref());
    let (session_token, csrf_token) = create_session_record(user.id, req).await;
    let _ = repository::execute(
        db::database(),
        "UPDATE ADMIN_USERS SET last_login_at = ?1, updated_at = ?1 WHERE id = ?2",
        vec![Value::from(now), Value::from(user.id)],
    )
    .await;

    write_audit_log(
        Some(user.id),
        Some(user.username.clone()),
        "login",
        "admin_session",
        None,
        "created admin session",
        None,
        ip,
        user_agent(req),
    )
    .await;

    Ok((user, session_token, csrf_token))
}

/// 通过邀请码注册后台账号。
pub async fn register(
    req: &Request,
    invitation_token: &str,
    username: &str,
    email: &str,
    password: &str,
) -> Result<(users::Model, String, String), &'static str> {
    let token_hash = hash_token(invitation_token);
    let Some(invitation) = find_invitation_by_token_hash(db::database(), &token_hash).await else {
        return Err("invitation_not_found");
    };

    let now = unix_timestamp();
    if invitation.status != InvitationStatus::Pending {
        return Err("invitation_invalid");
    }
    if invitation.expires_at <= now {
        let _ = repository::execute(
            db::database(),
            "UPDATE ADMIN_INVITATIONS SET status = ?1 WHERE id = ?2",
            vec![
                Value::from(InvitationStatus::Expired.as_str()),
                Value::from(invitation.id),
            ],
        )
        .await;
        return Err("invitation_expired");
    }

    let normalized_username = normalize_username(username)?;
    let normalized_email = normalize_email(email)?;
    let password_hash = hash_password(password).map_err(|_| "password_policy_failed")?;

    if find_user_by_username(db::database(), &normalized_username)
        .await
        .is_some()
    {
        return Err("username_taken");
    }
    if find_user_by_email(db::database(), &normalized_email)
        .await
        .is_some()
    {
        return Err("email_taken");
    }

    repository::execute(
        db::database(),
        "INSERT INTO ADMIN_USERS
         (username, email, password_hash, role, status, must_change_password, must_change_username, must_set_email, last_login_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, 0, 0, ?6, ?6, ?6)",
        vec![
            Value::from(normalized_username),
            Value::from(Some(normalized_email)),
            Value::from(password_hash),
            Value::from(invitation.role.as_str()),
            Value::from(AdminUserStatus::Active.as_str()),
            Value::from(now),
        ],
    )
    .await
    .map_err(|_| "user_creation_failed")?;

    let user_id = repository::last_insert_rowid(db::database())
        .await
        .map_err(|_| "user_creation_failed")?;
    let user = find_user_by_id(db::database(), user_id)
        .await
        .ok_or("user_creation_failed")?;

    let _ = repository::execute(
        db::database(),
        "UPDATE ADMIN_INVITATIONS
         SET status = ?1, consumed_at = ?2, consumed_by_user_id = ?3
         WHERE id = ?4",
        vec![
            Value::from(InvitationStatus::Consumed.as_str()),
            Value::from(Some(now)),
            Value::from(Some(user.id)),
            Value::from(invitation.id),
        ],
    )
    .await;

    write_audit_log(
        Some(user.id),
        Some(user.username.clone()),
        "register_by_invitation",
        "admin_user",
        Some(user.id.to_string()),
        "registered admin user with invitation",
        Some(serde_json::json!({
            "role": user.role.as_str(),
        })),
        client_ip(req),
        user_agent(req),
    )
    .await;

    let (session_token, csrf_token) = create_session_record(user.id, req).await;
    Ok((user, session_token, csrf_token))
}

/// 刷新当前会话的 CSRF token。
pub async fn rotate_csrf(session: &sessions::Model, req: &Request) -> String {
    let csrf_token = generate_token(32);
    let now = unix_timestamp();
    let _ = repository::execute(
        db::database(),
        "UPDATE ADMIN_SESSIONS
         SET csrf_token_hash = ?1, updated_at = ?2, last_seen_at = ?2, last_seen_ip = ?3
         WHERE id = ?4",
        vec![
            Value::from(hash_token(&csrf_token)),
            Value::from(now),
            Value::from(client_ip(req)),
            Value::from(session.id),
        ],
    )
    .await;
    csrf_token
}

/// 撤销当前会话。
pub async fn revoke_session(session: &sessions::Model, actor_user_id: Option<i64>, req: &Request) {
    let _ = repository::execute(
        db::database(),
        "UPDATE ADMIN_SESSIONS SET revoked_at = ?1, updated_at = ?1 WHERE id = ?2",
        vec![Value::from(unix_timestamp()), Value::from(session.id)],
    )
    .await;

    write_audit_log(
        actor_user_id,
        None,
        "logout",
        "admin_session",
        Some(session.id.to_string()),
        "revoked current admin session",
        None,
        client_ip(req),
        user_agent(req),
    )
    .await;
}

/// 创建后台会话记录。
pub async fn create_session_record(admin_user_id: i64, req: &Request) -> (String, String) {
    let session_token = generate_token(48);
    let csrf_token = generate_token(32);
    let now = unix_timestamp();
    let _ = repository::execute(
        db::database(),
        "INSERT INTO ADMIN_SESSIONS
         (admin_user_id, token_hash, csrf_token_hash, created_at, updated_at, expires_at, last_seen_at, revoked_at, created_ip, last_seen_ip, user_agent)
         VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?4, NULL, ?6, ?6, ?7)",
        vec![
            Value::from(admin_user_id),
            Value::from(hash_token(&session_token)),
            Value::from(hash_token(&csrf_token)),
            Value::from(now),
            Value::from(now + SESSION_TTL_SECONDS),
            Value::from(client_ip(req)),
            Value::from(user_agent(req)),
        ],
    )
    .await;
    (session_token, csrf_token)
}

pub fn client_ip(req: &Request) -> Option<String> {
    req.header::<String>("x-forwarded-for")
        .or_else(|| Some(req.remote_addr().to_string()))
}

pub fn user_agent(req: &Request) -> Option<String> {
    req.header::<String>("user-agent")
}

pub async fn find_user_by_identifier(
    db: &sea_orm::DatabaseConnection,
    identifier: &str,
) -> Option<users::Model> {
    let row = repository::query_one(
        db,
        "SELECT id, username, email, password_hash, role, status, must_change_password, must_change_username, must_set_email, last_login_at, created_at, updated_at
         FROM ADMIN_USERS
         WHERE username = ?1 OR email = ?1
         LIMIT 1",
        vec![Value::from(identifier.to_string())],
    )
    .await
    .ok()??;
    repository::map_user_row(&row).ok()
}

async fn find_user_by_username(
    db: &sea_orm::DatabaseConnection,
    username: &str,
) -> Option<users::Model> {
    let row = repository::query_one(
        db,
        "SELECT id, username, email, password_hash, role, status, must_change_password, must_change_username, must_set_email, last_login_at, created_at, updated_at
         FROM ADMIN_USERS WHERE username = ?1 LIMIT 1",
        vec![Value::from(username.to_string())],
    )
    .await
    .ok()??;
    repository::map_user_row(&row).ok()
}

async fn find_user_by_email(db: &sea_orm::DatabaseConnection, email: &str) -> Option<users::Model> {
    let row = repository::query_one(
        db,
        "SELECT id, username, email, password_hash, role, status, must_change_password, must_change_username, must_set_email, last_login_at, created_at, updated_at
         FROM ADMIN_USERS WHERE email = ?1 LIMIT 1",
        vec![Value::from(email.to_string())],
    )
    .await
    .ok()??;
    repository::map_user_row(&row).ok()
}

async fn find_invitation_by_token_hash(
    db: &sea_orm::DatabaseConnection,
    token_hash: &str,
) -> Option<invitations::Model> {
    let row = repository::query_one(
        db,
        "SELECT id, token_hash, role, status, invited_email, note, created_by_user_id, created_at, expires_at, consumed_at, consumed_by_user_id
         FROM ADMIN_INVITATIONS WHERE token_hash = ?1 LIMIT 1",
        vec![Value::from(token_hash.to_string())],
    )
    .await
    .ok()??;
    repository::map_invitation_row(&row).ok()
}

fn normalize_username(value: &str) -> Result<String, &'static str> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("username_required");
    }
    Ok(normalized)
}

fn normalize_email(value: &str) -> Result<String, &'static str> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("email_required");
    }
    Ok(normalized)
}

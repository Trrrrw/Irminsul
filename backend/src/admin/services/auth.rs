use salvo::Request;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, IntoActiveModel,
    QueryFilter,
};

use crate::{
    admin::{
        entities::{invitations, sessions, users},
        middlewares::auth::unix_timestamp,
        model::{AdminUserStatus, InvitationStatus},
        password::{hash_password, verify_password},
        services::{audit::write_audit_log, rate_limit},
        token::{generate_token, hash_token},
    },
    db,
};

pub const SESSION_TTL_SECONDS: i64 = 60 * 60 * 24 * 14;

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

    let pool = db::pool();
    let Some(user) = users::Entity::find()
        .filter(
            Condition::any()
                .add(users::Column::Username.eq(normalized_identifier.clone()))
                .add(users::Column::Email.eq(normalized_identifier.clone())),
        )
        .one(pool)
        .await
        .ok()
        .flatten()
    else {
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

    let mut active_user = user.clone().into_active_model();
    active_user.last_login_at = Set(Some(now));
    active_user.updated_at = Set(now);
    let _ = active_user.update(pool).await;

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

pub async fn register(
    req: &Request,
    invitation_token: &str,
    username: &str,
    email: &str,
    password: &str,
) -> Result<(users::Model, String, String), &'static str> {
    let pool = db::pool();
    let token_hash = hash_token(invitation_token);
    let Some(invitation) = invitations::Entity::find()
        .filter(invitations::Column::TokenHash.eq(token_hash))
        .one(pool)
        .await
        .ok()
        .flatten()
    else {
        return Err("invitation_not_found");
    };

    let now = unix_timestamp();
    if invitation.status != InvitationStatus::Pending {
        return Err("invitation_invalid");
    }
    if invitation.expires_at <= now {
        let mut expired = invitation.clone().into_active_model();
        expired.status = Set(InvitationStatus::Expired);
        let _ = expired.update(pool).await;
        return Err("invitation_expired");
    }

    let normalized_username = normalize_username(username)?;
    let normalized_email = normalize_email(email)?;
    let password_hash = hash_password(password).map_err(|_| "password_policy_failed")?;
    if users::Entity::find()
        .filter(users::Column::Username.eq(normalized_username.clone()))
        .one(pool)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        return Err("username_taken");
    }
    if users::Entity::find()
        .filter(users::Column::Email.eq(normalized_email.clone()))
        .one(pool)
        .await
        .ok()
        .flatten()
        .is_some()
    {
        return Err("email_taken");
    }
    let user = users::ActiveModel {
        username: Set(normalized_username),
        email: Set(Some(normalized_email)),
        password_hash: Set(password_hash),
        role: Set(invitation.role.clone()),
        status: Set(AdminUserStatus::Active),
        must_change_password: Set(false),
        must_change_username: Set(false),
        must_set_email: Set(false),
        last_login_at: Set(Some(now)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let user = user
        .insert(pool)
        .await
        .map_err(|_| "user_creation_failed")?;

    let mut invitation = invitation.into_active_model();
    invitation.status = Set(InvitationStatus::Consumed);
    invitation.consumed_at = Set(Some(now));
    invitation.consumed_by_user_id = Set(Some(user.id));
    let _ = invitation.update(pool).await;

    write_audit_log(
        Some(user.id),
        Some(user.username.clone()),
        "register_by_invitation",
        "admin_user",
        Some(user.id.to_string()),
        "registered admin user with invitation",
        Some(serde_json::json!({
            "role": user.role,
        })),
        client_ip(req),
        user_agent(req),
    )
    .await;

    let (session_token, csrf_token) = create_session_record(user.id, req).await;
    Ok((user, session_token, csrf_token))
}

pub async fn rotate_csrf(session: &sessions::Model, req: &Request) -> String {
    let csrf_token = generate_token(32);
    let mut active_session = session.clone().into_active_model();
    active_session.csrf_token_hash = Set(hash_token(&csrf_token));
    active_session.updated_at = Set(unix_timestamp());
    active_session.last_seen_at = Set(unix_timestamp());
    active_session.last_seen_ip = Set(client_ip(req));
    let _ = active_session.update(db::pool()).await;
    csrf_token
}

pub async fn revoke_session(session: &sessions::Model, actor_user_id: Option<i64>, req: &Request) {
    let mut active_session = session.clone().into_active_model();
    active_session.revoked_at = Set(Some(unix_timestamp()));
    let _ = active_session.update(db::pool()).await;

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

pub async fn create_session_record(admin_user_id: i64, req: &Request) -> (String, String) {
    let session_token = generate_token(48);
    let csrf_token = generate_token(32);
    let now = unix_timestamp();
    let session = sessions::ActiveModel {
        admin_user_id: Set(admin_user_id),
        token_hash: Set(hash_token(&session_token)),
        csrf_token_hash: Set(hash_token(&csrf_token)),
        created_at: Set(now),
        updated_at: Set(now),
        expires_at: Set(now + SESSION_TTL_SECONDS),
        last_seen_at: Set(now),
        revoked_at: Set(None),
        created_ip: Set(client_ip(req)),
        last_seen_ip: Set(client_ip(req)),
        user_agent: Set(user_agent(req)),
        ..Default::default()
    };
    let _ = session.insert(db::pool()).await;
    (session_token, csrf_token)
}

pub fn client_ip(req: &Request) -> Option<String> {
    req.header::<String>("x-forwarded-for")
        .or_else(|| Some(req.remote_addr().to_string()))
}

pub fn user_agent(req: &Request) -> Option<String> {
    req.header::<String>("user-agent")
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

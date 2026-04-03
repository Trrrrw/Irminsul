use sea_orm::Value;

use crate::admin::{
    db,
    dto::invitations::InvitationView,
    entities::invitations,
    model::{AdminRole, InvitationStatus},
    repository,
    services::audit::write_audit_log,
    token::{generate_token, hash_token},
};

/// 创建后台邀请码。
pub async fn create_invitation(
    actor_user_id: i64,
    role: AdminRole,
    note: Option<String>,
    expires_in_hours: Option<i64>,
    ip: Option<String>,
    user_agent: Option<String>,
) -> Result<(String, invitations::Model), &'static str> {
    if role == AdminRole::Owner {
        return Err("owner_invitation_forbidden");
    }

    let invitation_token = generate_token(32);
    let now = crate::admin::middlewares::auth::unix_timestamp();
    let expires_at = now + expires_in_hours.unwrap_or(24 * 7).clamp(1, 24 * 30) * 3600;

    repository::execute(
        db::database(),
        "INSERT INTO ADMIN_INVITATIONS
         (token_hash, role, status, invited_email, note, created_by_user_id, created_at, expires_at, consumed_at, consumed_by_user_id)
         VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, NULL, NULL)",
        vec![
            Value::from(hash_token(&invitation_token)),
            Value::from(role.as_str()),
            Value::from(InvitationStatus::Pending.as_str()),
            Value::from(note),
            Value::from(actor_user_id),
            Value::from(now),
            Value::from(expires_at),
        ],
    )
    .await
    .map_err(|_| "invitation_create_failed")?;

    let invitation = find_invitation_by_id(
        db::database(),
        repository::last_insert_rowid(db::database())
            .await
            .map_err(|_| "invitation_create_failed")?,
    )
    .await
    .ok_or("invitation_create_failed")?;

    write_audit_log(
        Some(actor_user_id),
        None,
        "create_invitation",
        "admin_invitation",
        Some(invitation.id.to_string()),
        "created admin invitation",
        Some(serde_json::json!({ "role": invitation.role.as_str() })),
        ip,
        user_agent,
    )
    .await;

    Ok((invitation_token, invitation))
}

/// 列出邀请码。
pub async fn list_invitations() -> Vec<InvitationView> {
    let Ok(rows) = repository::query_all(
        db::database(),
        "SELECT id, token_hash, role, status, invited_email, note, created_by_user_id, created_at, expires_at, consumed_at, consumed_by_user_id
         FROM ADMIN_INVITATIONS ORDER BY id DESC",
        Vec::new(),
    )
    .await
    else {
        return Vec::new();
    };

    rows.iter()
        .filter_map(|row| repository::map_invitation_row(row).ok())
        .map(|invitation| InvitationView {
            id: invitation.id,
            role: invitation.role,
            status: invitation.status,
            note: invitation.note,
            created_by_user_id: invitation.created_by_user_id,
            created_at: invitation.created_at,
            expires_at: invitation.expires_at,
            consumed_at: invitation.consumed_at,
            consumed_by_user_id: invitation.consumed_by_user_id,
        })
        .collect()
}

/// 撤销邀请码。
pub async fn revoke_invitation(
    invitation_id: i64,
    actor_user_id: i64,
    ip: Option<String>,
    user_agent: Option<String>,
) -> Result<(), &'static str> {
    let invitation = find_invitation_by_id(db::database(), invitation_id)
        .await
        .ok_or("invitation_not_found")?;
    if invitation.status != InvitationStatus::Pending {
        return Err("invitation_not_pending");
    }

    repository::execute(
        db::database(),
        "UPDATE ADMIN_INVITATIONS SET status = ?1 WHERE id = ?2",
        vec![
            Value::from(InvitationStatus::Revoked.as_str()),
            Value::from(invitation_id),
        ],
    )
    .await
    .map_err(|_| "invitation_revoke_failed")?;

    write_audit_log(
        Some(actor_user_id),
        None,
        "revoke_invitation",
        "admin_invitation",
        Some(invitation_id.to_string()),
        "revoked admin invitation",
        None,
        ip,
        user_agent,
    )
    .await;

    Ok(())
}

pub async fn find_invitation_by_id(
    db: &sea_orm::DatabaseConnection,
    invitation_id: i64,
) -> Option<invitations::Model> {
    let row = repository::query_one(
        db,
        "SELECT id, token_hash, role, status, invited_email, note, created_by_user_id, created_at, expires_at, consumed_at, consumed_by_user_id
         FROM ADMIN_INVITATIONS WHERE id = ?1 LIMIT 1",
        vec![Value::from(invitation_id)],
    )
    .await
    .ok()??;
    repository::map_invitation_row(&row).ok()
}

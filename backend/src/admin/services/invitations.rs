use std::str::FromStr;

use crate::admin::{
    db,
    dto::invitations::InvitationView,
    entities::invitations,
    model::{AdminRole, InvitationStatus},
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
    let conn = db::database().connect().map_err(|_| "db_unavailable")?;

    conn.execute(
        "INSERT INTO ADMIN_INVITATIONS
         (token_hash, role, status, invited_email, note, created_by_user_id, created_at, expires_at, consumed_at, consumed_by_user_id)
         VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, NULL, NULL)",
        turso::params![
            hash_token(&invitation_token),
            role.as_str(),
            InvitationStatus::Pending.as_str(),
            note,
            actor_user_id,
            now,
            expires_at,
        ],
    )
    .await
    .map_err(|_| "invitation_create_failed")?;

    let invitation = find_invitation_by_id(&conn, conn.last_insert_rowid())
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
    let Ok(conn) = db::database().connect() else {
        return Vec::new();
    };
    let Ok(mut rows) = conn
        .query(
            "SELECT id, token_hash, role, status, invited_email, note, created_by_user_id, created_at, expires_at, consumed_at, consumed_by_user_id
             FROM ADMIN_INVITATIONS ORDER BY id DESC",
            (),
        )
        .await
    else {
        return Vec::new();
    };

    let mut values = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        if let Ok(invitation) = map_invitation_row(&row) {
            values.push(InvitationView {
                id: invitation.id,
                role: invitation.role,
                status: invitation.status,
                note: invitation.note,
                created_by_user_id: invitation.created_by_user_id,
                created_at: invitation.created_at,
                expires_at: invitation.expires_at,
                consumed_at: invitation.consumed_at,
                consumed_by_user_id: invitation.consumed_by_user_id,
            });
        }
    }
    values
}

/// 撤销邀请码。
pub async fn revoke_invitation(
    invitation_id: i64,
    actor_user_id: i64,
    ip: Option<String>,
    user_agent: Option<String>,
) -> Result<(), &'static str> {
    let conn = db::database().connect().map_err(|_| "db_unavailable")?;
    let invitation = find_invitation_by_id(&conn, invitation_id)
        .await
        .ok_or("invitation_not_found")?;
    if invitation.status != InvitationStatus::Pending {
        return Err("invitation_not_pending");
    }

    conn.execute(
        "UPDATE ADMIN_INVITATIONS SET status = ?1 WHERE id = ?2",
        turso::params![InvitationStatus::Revoked.as_str(), invitation_id],
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

async fn find_invitation_by_id(
    conn: &turso::Connection,
    invitation_id: i64,
) -> Option<invitations::Model> {
    let mut rows = conn
        .query(
            "SELECT id, token_hash, role, status, invited_email, note, created_by_user_id, created_at, expires_at, consumed_at, consumed_by_user_id
             FROM ADMIN_INVITATIONS WHERE id = ?1 LIMIT 1",
            turso::params![invitation_id],
        )
        .await
        .ok()?;
    let row = rows.next().await.ok().flatten()?;
    map_invitation_row(&row).ok()
}

fn map_invitation_row(row: &turso::Row) -> Result<invitations::Model, String> {
    Ok(invitations::Model {
        id: row.get(0).map_err(|error| error.to_string())?,
        token_hash: row.get(1).map_err(|error| error.to_string())?,
        role: AdminRole::from_str(&row.get::<String>(2).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?,
        status: InvitationStatus::from_str(
            &row.get::<String>(3).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?,
        invited_email: row.get(4).map_err(|error| error.to_string())?,
        note: row.get(5).map_err(|error| error.to_string())?,
        created_by_user_id: row.get(6).map_err(|error| error.to_string())?,
        created_at: row.get(7).map_err(|error| error.to_string())?,
        expires_at: row.get(8).map_err(|error| error.to_string())?,
        consumed_at: row.get(9).map_err(|error| error.to_string())?,
        consumed_by_user_id: row.get(10).map_err(|error| error.to_string())?,
    })
}

use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, IntoActiveModel};

use crate::admin::{
    db,
    dto::invitations::InvitationView,
    entities::invitations,
    model::{AdminRole, InvitationStatus},
    services::audit::write_audit_log,
    token::{generate_token, hash_token},
};

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
    let expires_at = crate::admin::middlewares::auth::unix_timestamp()
        + expires_in_hours.unwrap_or(24 * 7).clamp(1, 24 * 30) * 3600;
    let invitation = invitations::ActiveModel {
        token_hash: Set(hash_token(&invitation_token)),
        role: Set(role),
        status: Set(InvitationStatus::Pending),
        note: Set(note),
        created_by_user_id: Set(actor_user_id),
        created_at: Set(crate::admin::middlewares::auth::unix_timestamp()),
        expires_at: Set(expires_at),
        consumed_at: Set(None),
        consumed_by_user_id: Set(None),
        ..Default::default()
    };
    let invitation = invitation
        .insert(db::pool())
        .await
        .map_err(|_| "invitation_create_failed")?;

    write_audit_log(
        Some(actor_user_id),
        None,
        "create_invitation",
        "admin_invitation",
        Some(invitation.id.to_string()),
        "created admin invitation",
        Some(serde_json::json!({
            "role": invitation.role,
        })),
        ip,
        user_agent,
    )
    .await;

    Ok((invitation_token, invitation))
}

pub async fn list_invitations() -> Vec<InvitationView> {
    invitations::Entity::find()
        .all(db::pool())
        .await
        .unwrap_or_default()
        .into_iter()
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

pub async fn revoke_invitation(
    invitation_id: i64,
    actor_user_id: i64,
    ip: Option<String>,
    user_agent: Option<String>,
) -> Result<(), &'static str> {
    let Some(invitation) = invitations::Entity::find_by_id(invitation_id)
        .one(db::pool())
        .await
        .ok()
        .flatten()
    else {
        return Err("invitation_not_found");
    };
    if invitation.status != InvitationStatus::Pending {
        return Err("invitation_not_pending");
    }

    let mut active = invitation.into_active_model();
    active.status = Set(InvitationStatus::Revoked);
    active
        .update(db::pool())
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

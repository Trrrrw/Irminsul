use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter,
};

use crate::{
    admin::{
        dto::users::{AdminUserView, UpdateSelfProfileRequest},
        entities::users,
        model::{AdminRole, AdminUserStatus},
        password::{hash_password, verify_password},
        services::audit::write_audit_log,
    },
    db,
};

pub async fn list_users() -> Vec<AdminUserView> {
    users::Entity::find()
        .all(db::pool())
        .await
        .unwrap_or_default()
        .into_iter()
        .map(AdminUserView::from)
        .collect()
}

pub async fn set_user_enabled(
    target_user_id: i64,
    enabled: bool,
    actor_user_id: i64,
    actor_role: &AdminRole,
    ip: Option<String>,
    user_agent: Option<String>,
) -> Result<users::Model, &'static str> {
    let Some(user) = users::Entity::find_by_id(target_user_id)
        .one(db::pool())
        .await
        .ok()
        .flatten()
    else {
        return Err("user_not_found");
    };
    if !enabled && actor_user_id == target_user_id {
        return Err("cannot_disable_self");
    }
    if user.role == AdminRole::Owner && actor_role != &AdminRole::Owner {
        return Err("owner_management_forbidden");
    }
    if !enabled && user.role == AdminRole::Owner && owner_count().await <= 1 {
        return Err("cannot_disable_last_owner");
    }

    let mut active = user.into_active_model();
    active.status = Set(if enabled {
        AdminUserStatus::Active
    } else {
        AdminUserStatus::Disabled
    });
    active.updated_at = Set(crate::admin::middlewares::auth::unix_timestamp());
    let user = active
        .update(db::pool())
        .await
        .map_err(|_| "user_status_update_failed")?;

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
            "role": user.role,
        })),
        ip,
        user_agent,
    )
    .await;

    Ok(user)
}

pub async fn update_self_profile(
    current_user_id: i64,
    payload: UpdateSelfProfileRequest,
) -> Result<users::Model, &'static str> {
    let Some(user) = users::Entity::find_by_id(current_user_id)
        .one(db::pool())
        .await
        .ok()
        .flatten()
    else {
        return Err("user_not_found");
    };

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

    let mut active = user.clone().into_active_model();
    active.username = Set(new_username);
    active.email = Set(new_email.clone());

    if let Some(new_password) = payload.new_password.as_deref() {
        if !user.must_change_password {
            let Some(current_password) = payload.current_password.as_deref() else {
                return Err("current_password_required");
            };
            if !verify_password(current_password, &user.password_hash) {
                return Err("current_password_invalid");
            }
        }
        active.password_hash =
            Set(hash_password(new_password).map_err(|_| "password_policy_failed")?);
        active.must_change_password = Set(false);
    } else if user.must_change_password {
        return Err("password_change_required");
    }

    if user.must_change_username {
        active.must_change_username = Set(false);
    }
    if user.must_set_email && new_email.is_some() {
        active.must_set_email = Set(false);
    }
    active.updated_at = Set(crate::admin::middlewares::auth::unix_timestamp());

    active
        .update(db::pool())
        .await
        .map_err(|_| "profile_update_failed")
}

fn normalize_username(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_optional_email(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

async fn owner_count() -> u64 {
    users::Entity::find()
        .filter(users::Column::Role.eq(AdminRole::Owner))
        .count(db::pool())
        .await
        .unwrap_or(0)
}

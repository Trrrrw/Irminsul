use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, ExecResult, QueryResult, Statement, Value,
};

use crate::{
    admin::{
        entities::{audit_logs, invitations, sessions, users},
        model::{AdminRole, AdminUserStatus, InvitationStatus},
    },
    audit::model::AuditActorType,
};

fn statement(sql: &str, values: Vec<Value>) -> Statement {
    Statement::from_sql_and_values(DbBackend::Sqlite, sql, values)
}

pub async fn execute(
    db: &DatabaseConnection,
    sql: &str,
    values: Vec<Value>,
) -> Result<ExecResult, sea_orm::DbErr> {
    db.execute(statement(sql, values)).await
}

pub async fn query_one(
    db: &DatabaseConnection,
    sql: &str,
    values: Vec<Value>,
) -> Result<Option<QueryResult>, sea_orm::DbErr> {
    db.query_one(statement(sql, values)).await
}

pub async fn query_all(
    db: &DatabaseConnection,
    sql: &str,
    values: Vec<Value>,
) -> Result<Vec<QueryResult>, sea_orm::DbErr> {
    db.query_all(statement(sql, values)).await
}

pub async fn last_insert_rowid(db: &DatabaseConnection) -> Result<i64, sea_orm::DbErr> {
    let row = query_one(db, "SELECT last_insert_rowid() AS id", Vec::new()).await?;
    Ok(row
        .as_ref()
        .and_then(|value| value.try_get::<i64>("", "id").ok())
        .unwrap_or_default())
}

pub fn map_user_row(row: &QueryResult) -> Result<users::Model, String> {
    Ok(users::Model {
        id: row.try_get("", "id").map_err(|error| error.to_string())?,
        username: row
            .try_get("", "username")
            .map_err(|error| error.to_string())?,
        email: row
            .try_get("", "email")
            .map_err(|error| error.to_string())?,
        password_hash: row
            .try_get("", "password_hash")
            .map_err(|error| error.to_string())?,
        role: row
            .try_get::<String>("", "role")
            .map_err(|error| error.to_string())?
            .parse::<AdminRole>()
            .map_err(|error| error.to_string())?,
        status: row
            .try_get::<String>("", "status")
            .map_err(|error| error.to_string())?
            .parse::<AdminUserStatus>()
            .map_err(|error| error.to_string())?,
        must_change_password: row
            .try_get::<i64>("", "must_change_password")
            .map_err(|error| error.to_string())?
            != 0,
        must_change_username: row
            .try_get::<i64>("", "must_change_username")
            .map_err(|error| error.to_string())?
            != 0,
        must_set_email: row
            .try_get::<i64>("", "must_set_email")
            .map_err(|error| error.to_string())?
            != 0,
        last_login_at: row
            .try_get("", "last_login_at")
            .map_err(|error| error.to_string())?,
        created_at: row
            .try_get("", "created_at")
            .map_err(|error| error.to_string())?,
        updated_at: row
            .try_get("", "updated_at")
            .map_err(|error| error.to_string())?,
    })
}

pub fn map_session_row(row: &QueryResult) -> Result<sessions::Model, String> {
    Ok(sessions::Model {
        id: row.try_get("", "id").map_err(|error| error.to_string())?,
        admin_user_id: row
            .try_get("", "admin_user_id")
            .map_err(|error| error.to_string())?,
        token_hash: row
            .try_get("", "token_hash")
            .map_err(|error| error.to_string())?,
        csrf_token_hash: row
            .try_get("", "csrf_token_hash")
            .map_err(|error| error.to_string())?,
        created_at: row
            .try_get("", "created_at")
            .map_err(|error| error.to_string())?,
        updated_at: row
            .try_get("", "updated_at")
            .map_err(|error| error.to_string())?,
        expires_at: row
            .try_get("", "expires_at")
            .map_err(|error| error.to_string())?,
        last_seen_at: row
            .try_get("", "last_seen_at")
            .map_err(|error| error.to_string())?,
        revoked_at: row
            .try_get("", "revoked_at")
            .map_err(|error| error.to_string())?,
        created_ip: row
            .try_get("", "created_ip")
            .map_err(|error| error.to_string())?,
        last_seen_ip: row
            .try_get("", "last_seen_ip")
            .map_err(|error| error.to_string())?,
        user_agent: row
            .try_get("", "user_agent")
            .map_err(|error| error.to_string())?,
    })
}

pub fn map_invitation_row(row: &QueryResult) -> Result<invitations::Model, String> {
    Ok(invitations::Model {
        id: row.try_get("", "id").map_err(|error| error.to_string())?,
        token_hash: row
            .try_get("", "token_hash")
            .map_err(|error| error.to_string())?,
        role: row
            .try_get::<String>("", "role")
            .map_err(|error| error.to_string())?
            .parse::<AdminRole>()
            .map_err(|error| error.to_string())?,
        status: row
            .try_get::<String>("", "status")
            .map_err(|error| error.to_string())?
            .parse::<InvitationStatus>()
            .map_err(|error| error.to_string())?,
        invited_email: row
            .try_get("", "invited_email")
            .map_err(|error| error.to_string())?,
        note: row.try_get("", "note").map_err(|error| error.to_string())?,
        created_by_user_id: row
            .try_get("", "created_by_user_id")
            .map_err(|error| error.to_string())?,
        created_at: row
            .try_get("", "created_at")
            .map_err(|error| error.to_string())?,
        expires_at: row
            .try_get("", "expires_at")
            .map_err(|error| error.to_string())?,
        consumed_at: row
            .try_get("", "consumed_at")
            .map_err(|error| error.to_string())?,
        consumed_by_user_id: row
            .try_get("", "consumed_by_user_id")
            .map_err(|error| error.to_string())?,
    })
}

pub fn map_audit_log_row(row: &QueryResult) -> Result<audit_logs::Model, String> {
    Ok(audit_logs::Model {
        id: row.try_get("", "id").map_err(|error| error.to_string())?,
        actor_type: row
            .try_get::<String>("", "actor_type")
            .map_err(|error| error.to_string())?
            .parse::<AuditActorType>()
            .map_err(|error| error.to_string())?,
        actor_user_id: row
            .try_get("", "actor_user_id")
            .map_err(|error| error.to_string())?,
        actor_label: row
            .try_get("", "actor_label")
            .map_err(|error| error.to_string())?,
        action: row
            .try_get("", "action")
            .map_err(|error| error.to_string())?,
        target_type: row
            .try_get("", "target_type")
            .map_err(|error| error.to_string())?,
        target_id: row
            .try_get("", "target_id")
            .map_err(|error| error.to_string())?,
        summary: row
            .try_get("", "summary")
            .map_err(|error| error.to_string())?,
        metadata_json: row
            .try_get("", "metadata_json")
            .map_err(|error| error.to_string())?,
        ip: row.try_get("", "ip").map_err(|error| error.to_string())?,
        user_agent: row
            .try_get("", "user_agent")
            .map_err(|error| error.to_string())?,
        created_at: row
            .try_get("", "created_at")
            .map_err(|error| error.to_string())?,
    })
}

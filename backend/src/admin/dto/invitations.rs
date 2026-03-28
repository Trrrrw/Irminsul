use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

use crate::admin::model::{AdminRole, InvitationStatus};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInvitationRequest {
    pub role: AdminRole,
    pub note: Option<String>,
    pub expires_in_hours: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationView {
    pub id: i64,
    pub role: AdminRole,
    pub status: InvitationStatus,
    pub note: Option<String>,
    pub created_by_user_id: i64,
    pub created_at: i64,
    pub expires_at: i64,
    pub consumed_at: Option<i64>,
    pub consumed_by_user_id: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateInvitationResponse {
    pub invitation_token: String,
    pub invitation: InvitationView,
}

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

/// 管理后台用户角色。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdminRole {
    Owner,
    Admin,
    Editor,
    Viewer,
}

impl AdminRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Viewer => "viewer",
        }
    }
}

impl Display for AdminRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AdminRole {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "editor" => Ok(Self::Editor),
            "viewer" => Ok(Self::Viewer),
            _ => Err(format!("unsupported admin role: {value}")),
        }
    }
}

/// 管理后台账号状态。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdminUserStatus {
    Active,
    Disabled,
}

impl AdminUserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}

impl Display for AdminUserStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AdminUserStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            _ => Err(format!("unsupported admin user status: {value}")),
        }
    }
}

/// 邀请码生命周期状态。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InvitationStatus {
    Pending,
    Consumed,
    Revoked,
    Expired,
}

impl InvitationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Consumed => "consumed",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl Display for InvitationStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for InvitationStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "consumed" => Ok(Self::Consumed),
            "revoked" => Ok(Self::Revoked),
            "expired" => Ok(Self::Expired),
            _ => Err(format!("unsupported invitation status: {value}")),
        }
    }
}

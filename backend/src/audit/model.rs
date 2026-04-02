use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// 审计日志中的操作者类型。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditActorType {
    AdminUser,
    Plugin,
    System,
    Scheduler,
}

impl AuditActorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AdminUser => "admin_user",
            Self::Plugin => "plugin",
            Self::System => "system",
            Self::Scheduler => "scheduler",
        }
    }
}

impl Display for AuditActorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AuditActorType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "admin_user" => Ok(Self::AdminUser),
            "plugin" => Ok(Self::Plugin),
            "system" => Ok(Self::System),
            "scheduler" => Ok(Self::Scheduler),
            _ => Err(format!("unsupported audit actor type: {value}")),
        }
    }
}

use serde::{Serialize, Deserialize};


/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub status: UserStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Blocked,
}

impl UserStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Active" => Self::Active,
            "Inactive" => Self::Inactive,
            "Blocked" => Self::Blocked,
            _ => Self::Inactive,
        }
    }
}

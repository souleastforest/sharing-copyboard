use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub device_id: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
}
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)] // 添加 sqlx::FromRow
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub username: String,  // 从 Option<String> 改为 String
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: Option<String>,
    pub username: String,
    pub created_at: i64,
    pub updated_at: i64,
}
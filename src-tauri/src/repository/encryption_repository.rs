use sqlx::SqlitePool;
use crate::error::AppError;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)] // 添加 sqlx::FromRow
pub struct EncryptionKey {
    pub id: String,
    pub user_id: String,
    pub key_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: i64,
}

pub struct EncryptionRepository;

impl EncryptionRepository {
    pub async fn save(pool: &SqlitePool, key: &EncryptionKey) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO encryption_keys (id, user_id, key_data, nonce, created_at)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&key.id)
        .bind(&key.user_id)
        .bind(&key.key_data)
        .bind(&key.nonce)
        .bind(key.created_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn find_by_user_id(pool: &SqlitePool, user_id: &str) -> Result<Option<EncryptionKey>, AppError> {
        let key = sqlx::query_as::<_, EncryptionKey>(
            "SELECT id, user_id, key_data, nonce, created_at
             FROM encryption_keys WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(key)
    }
    
    pub async fn create_for_user(pool: &SqlitePool, user_id: &str) -> Result<EncryptionKey, AppError> {
        // 检查是否已存在
        let existing = Self::find_by_user_id(pool, user_id).await?;
        if existing.is_some() {
            return Err(AppError::InvalidData("用户已有加密密钥".to_string()));
        }
        
        // 生成新密钥
        use crate::util::crypto;
        let key_data = crypto::generate_encryption_key().to_vec();
        let nonce = crypto::generate_nonce().to_vec();
        
        let id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let key = EncryptionKey {
            id,
            user_id: user_id.to_string(),
            key_data,
            nonce,
            created_at: now,
        };
        
        Self::save(pool, &key).await?;
        
        Ok(key)
    }
}
use sqlx::SqlitePool;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::entity::user::{User, UserProfile};
use crate::repository::user_repository::UserRepository;
use crate::repository::session_repository::SessionRepository;
use crate::error::AppError;
use crate::util::crypto;

pub struct UserService;

impl UserService {
    pub async fn register(
        pool: &SqlitePool, 
        email: &str, 
        password: &str, 
        verification_code: &str
    ) -> Result<User, AppError> {
        // 验证验证码
        let is_valid = Self::verify_code(pool, email, verification_code).await?;
        
        if !is_valid {
            return Err(AppError::InvalidData("验证码无效".to_string()));
        }
        
        // 检查邮箱是否已存在
        let existing_user = UserRepository::find_by_email(pool, email).await?;
        
        if existing_user.is_some() {
            return Err(AppError::InvalidData("邮箱已存在".to_string()));
        }
        
        // 哈希密码
        let password_hash = crypto::hash_password(password)
            .map_err(|e| AppError::CryptoError(e))?;
        
        let id = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let username = email.split('@').next().unwrap_or("user");
        
        let user = User {
            id: id.clone(),
            email: Option::from(email.to_string()),
            username: username.to_string(),
            created_at: now,
            updated_at: now,
        };
        
        // 保存用户
        UserRepository::save(pool, &user, &password_hash).await?;
        
        // 删除已使用的验证码
        sqlx::query!("DELETE FROM verification_codes WHERE email = ?", email)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(user)
    }
    
    pub async fn get_profile(pool: &SqlitePool, user_id: &str) -> Result<UserProfile, AppError> {
        let user = match UserRepository::find_by_id(pool, user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFound("用户不存在".to_string())),
        };
        
        let device_count = SessionRepository::count_by_user_id(pool, user_id).await?;
        
        Ok(UserProfile {
            id: user.id,
            email: user.email,
            username: user.username,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }
    
    pub async fn update_profile(
        pool: &SqlitePool, 
        user_id: &str, 
        username: &str, 
        email: &str
    ) -> Result<UserProfile, AppError> {
        let user = match UserRepository::find_by_id(pool, user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFound("用户不存在".to_string())),
        };
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        sqlx::query(
            "UPDATE users SET
             email = ?,
             username = ?,
             updated_at = ?
             WHERE id = ?"
        )
        .bind(email)
        .bind(username)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let updated_user = User {
            id: user.id,
            email: Some(email.to_string()),
            username: username.to_string(),
            created_at: user.created_at,
            updated_at: now,
        };
        
        let device_count = SessionRepository::count_by_user_id(pool, user_id).await?;
        
        Ok(UserProfile {
            id: updated_user.id,
            email: updated_user.email,
            username: updated_user.username,
            created_at: updated_user.created_at,
            updated_at: updated_user.updated_at,
        })
    }
    
    // 验证验证码
    async fn verify_code(pool: &SqlitePool, email: &str, code: &str) -> Result<bool, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let result = sqlx::query!(
            "SELECT code FROM verification_codes WHERE email = ? AND expires_at > ?",
            email, now
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        match result {
            Some(row) => Ok(row.code == code),
            None => Ok(false),
        }
    }
    
    // 其他用户相关方法...
}
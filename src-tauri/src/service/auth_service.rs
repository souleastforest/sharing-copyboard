use sqlx::SqlitePool;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::entity::user::User;
use crate::entity::session::Session;
use crate::repository::user_repository::UserRepository;
use crate::repository::session_repository::SessionRepository;
use crate::error::AppError;
use crate::util::crypto;

pub struct AuthService;

impl AuthService {
    pub async fn login(pool: &SqlitePool, email: &str, password: &str, device_id: &str) -> Result<Session, AppError> {
        // 查找用户
        let user = match UserRepository::find_by_email(pool, email).await? {
            Some(user) => user,
            None => return Err(AppError::NotFound("用户不存在".to_string())),
        };
        
        // 获取密码哈希
        let password_hash = sqlx::query!(
            "SELECT password_hash FROM users WHERE id = ?",
            user.id
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .password_hash;
        
        // 验证密码
        let is_valid = crypto::verify_password(&password_hash, password)
            .map_err(|e| AppError::CryptoError(e))?;
        
        if !is_valid {
            return Err(AppError::InvalidCredentials);
        }
        
        // 创建会话
        let token = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expires_at = now + 30 * 24 * 60 * 60; // 30天过期
        
        let session = Session {
            token: token.clone(),
            user_id: user.id,
            device_id: Some(device_id.to_string()),
            created_at: now,
            expires_at,
        };
        
        // 保存会话
        SessionRepository::save(pool, &session).await?;
        
        Ok(session)
    }
    
    pub async fn logout(pool: &SqlitePool, token: &str) -> Result<(), AppError> {
        SessionRepository::delete_by_token(pool, token).await
    }
    
    pub async fn verify_session(pool: &SqlitePool, token: &str) -> Result<User, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        // 查找有效会话
        let session = match SessionRepository::find_by_token(pool, token).await? {
            Some(session) if session.expires_at > now => session,
            Some(_) => return Err(AppError::InvalidData("会话已过期".to_string())),
            None => return Err(AppError::NotFound("会话不存在".to_string())),
        };
        
        // 获取用户信息
        let user = match UserRepository::find_by_id(pool, &session.user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFound("用户不存在".to_string())),
        };
        
        Ok(user)
    }
    
    pub async fn change_password(
        pool: &SqlitePool, 
        user_id: &str, 
        old_password: &str, 
        new_password: &str
    ) -> Result<(), AppError> {
        // 获取当前密码哈希
        let password_hash = sqlx::query!(
            "SELECT password_hash FROM users WHERE id = ?", 
            user_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?
        .password_hash;
        
        // 验证旧密码
        let is_valid = crypto::verify_password(&password_hash, old_password)
            .map_err(|e| AppError::CryptoError(e))?;
        
        if !is_valid {
            return Err(AppError::InvalidData("旧密码不正确".to_string()));
        }
        
        // 哈希新密码
        let new_password_hash = crypto::hash_password(new_password)
            .map_err(|e| AppError::CryptoError(e))?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        // 更新密码
        sqlx::query(
            "UPDATE users SET
             password_hash = ?,
             updated_at = ?
             WHERE id = ?"
        )
        .bind(&new_password_hash)
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn request_password_reset(pool: &SqlitePool, email: &str) -> Result<String, AppError> {
        // 检查用户是否存在
        let user = match UserRepository::find_by_email(pool, email).await? {
            Some(user) => user,
            None => return Err(AppError::NotFound("用户不存在".to_string())),
        };
        
        // 生成重置令牌
        let token = Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expires_at = now + 24 * 60 * 60; // 24小时过期
        
        // 删除旧的重置令牌
        sqlx::query!("DELETE FROM password_resets WHERE email = ?", email)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        // 创建新的重置令牌
        sqlx::query(
            "INSERT INTO password_resets (email, token, user_id, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(email)
        .bind(&token)
        .bind(&user.id)
        .bind(now)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(token)
    }
    
    pub async fn reset_password(
        pool: &SqlitePool, 
        email: &str, 
        reset_token: &str, 
        new_password: &str
    ) -> Result<(), AppError> {
        // 验证重置令牌
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let reset = sqlx::query!(
            "SELECT user_id FROM password_resets WHERE email = ? AND token = ? AND expires_at > ?",
            email, reset_token, now
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let user_id = match reset {
            Some(reset) => reset.user_id,
            None => return Err(AppError::InvalidData("无效或已过期的重置令牌".to_string())),
        };
        
        // 哈希新密码
        let new_password_hash = crypto::hash_password(new_password)
            .map_err(|e| AppError::CryptoError(e))?;
        
        // 更新密码
        sqlx::query(
            "UPDATE users SET
             password_hash = ?,
             updated_at = ?
             WHERE id = ?"
        )
        .bind(&new_password_hash)
        .bind(now)
        .bind(&user_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        // 删除使用过的重置令牌
        sqlx::query!("DELETE FROM password_resets WHERE email = ?", email)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
}
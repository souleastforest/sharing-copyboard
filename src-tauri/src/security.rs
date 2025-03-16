use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce
};
use argon2::{self, password_hash::{PasswordHasher, SaltString, PasswordHash, PasswordVerifier}};
use argon2::Argon2;
use rand::{Rng, thread_rng};
// 删除重复的导入
// use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::DbError;

// 用户认证相关结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub device_id: String,
    pub created_at: i64,
    pub expires_at: i64,
}

// 加密相关函数

// 生成随机密钥
pub fn generate_encryption_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    thread_rng().fill(&mut key);
    key
}

// 生成随机IV (Initialization Vector)
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    thread_rng().fill(&mut nonce);
    nonce
}

// 加密数据
pub fn encrypt_data(data: &[u8], encryption_key: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
    let key = Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    cipher.encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))
}

// 解密数据
pub fn decrypt_data(encrypted_data: &[u8], encryption_key: &[u8], nonce: &[u8; 12]) -> Result<String, String> {
    let key = Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    let decrypted = cipher.decrypt(nonce, encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    String::from_utf8(decrypted)
        .map_err(|e| format!("Invalid UTF-8 sequence: {}", e))
}

// 密码哈希与验证

// 生成密码哈希
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut thread_rng());
    let argon2 = Argon2::default();
    
    argon2.hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("Password hashing failed: {}", e))
}

// 验证密码
pub fn verify_password(hash: &str, password: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| format!("Invalid password hash: {}", e))?;
    
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

// 生成随机盐值不再需要，使用 SaltString::generate 代替
// fn generate_salt() -> [u8; 16] {
//     let mut salt = [0u8; 16];
//     thread_rng().fill(&mut salt);
//     salt
// }

// 用户认证相关数据库操作

// 创建用户
pub async fn create_user(pool: &SqlitePool, email: &str, password: &str) -> Result<User, DbError> {
    // 检查邮箱是否已存在
    let existing = sqlx::query!("SELECT id FROM users WHERE email = ?", email)
        .fetch_optional(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    if existing.is_some() {
        return Err(DbError::InvalidData("Email already exists".to_string()));
    }
    
    // 哈希密码
    let password_hash = hash_password(password)
        .map_err(|e| DbError::QueryError(e))?;
    
    let id = Uuid::new_v4().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // 插入用户记录
    sqlx::query(
        "
        INSERT INTO users (id, email, password_hash, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        "
    )
    .bind(&id)
    .bind(email)
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    Ok(User {
        id,
        email: email.to_string(),
        created_at: now,
        updated_at: now,
    })
}

// 用户登录
pub async fn login_user(pool: &SqlitePool, email: &str, password: &str, device_id: &str) -> Result<Session, DbError> {
    // 查找用户
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE email = ?",
        email
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    let user = match user {
        Some(user) => user,
        None => return Err(DbError::NotFound),
    };
    
    // 验证密码
    let is_valid = verify_password(&user.password_hash, password)
        .map_err(|e| DbError::QueryError(e))?;
    
    if !is_valid {
        return Err(DbError::InvalidData("Invalid password".to_string()));
    }
    
    // 创建会话
    let token = Uuid::new_v4().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let expires_at = now + 30 * 24 * 60 * 60; // 30天过期
    
    sqlx::query(
        "
        INSERT INTO sessions (token, user_id, device_id, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?)
        "
    )
    .bind(&token)
    .bind(&user.id)
    .bind(device_id)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    Ok(Session {
        token,
        user_id: user.id,
        device_id: device_id.to_string(),
        created_at: now,
        expires_at,
    })
}

// 验证会话
pub async fn verify_session(pool: &SqlitePool, token: &str) -> Result<User, DbError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // 查找有效会话
    let session = sqlx::query!(
        "SELECT user_id FROM sessions WHERE token = ? AND expires_at > ?",
        token, now
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    let user_id = match session {
        Some(session) => session.user_id,
        None => return Err(DbError::NotFound),
    };
    
    // 修改查询，使用 TEXT 而不是 TIMESTAMP
    let user = sqlx::query!(
        "SELECT id, email, created_at as \"created_at: i64\", updated_at as \"updated_at: i64\" FROM users WHERE id = ?",
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    match user {
        Some(user) => Ok(User {
            id: user.id,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }),
        None => Err(DbError::NotFound),
    }
}

// 注销会话
pub async fn logout_session(pool: &SqlitePool, token: &str) -> Result<(), DbError> {
    sqlx::query!("DELETE FROM sessions WHERE token = ?", token)
        .execute(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    Ok(())
}

// 修改密码
pub async fn change_password(pool: &SqlitePool, user_id: &str, old_password: &str, new_password: &str) -> Result<(), DbError> {
    // 获取当前密码哈希
    let user = sqlx::query!("SELECT password_hash FROM users WHERE id = ?", user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    let user = match user {
        Some(user) => user,
        None => return Err(DbError::NotFound),
    };
    
    // 验证旧密码
    let is_valid = verify_password(&user.password_hash, old_password)
        .map_err(|e| DbError::QueryError(e))?;
    
    if !is_valid {
        return Err(DbError::InvalidData("Invalid old password".to_string()));
    }
    
    // 哈希新密码
    let new_password_hash = hash_password(new_password)
        .map_err(|e| DbError::QueryError(e))?;
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // 更新密码
    sqlx::query(
        "
        UPDATE users SET
        password_hash = ?,
        updated_at = ?
        WHERE id = ?
        "
    )
    .bind(&new_password_hash)
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    Ok(())
}

// 重置密码（忘记密码流程）
pub async fn reset_password(pool: &SqlitePool, email: &str, reset_token: &str, new_password: &str) -> Result<(), DbError> {
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
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    let user_id = match reset {
        Some(reset) => reset.user_id,
        None => return Err(DbError::InvalidData("Invalid or expired reset token".to_string())),
    };
    
    // 哈希新密码
    let new_password_hash = hash_password(new_password)
        .map_err(|e| DbError::QueryError(e))?;
    
    // 更新密码
    sqlx::query(
        "
        UPDATE users SET
        password_hash = ?,
        updated_at = ?
        WHERE id = ?
        "
    )
    .bind(&new_password_hash)
    .bind(now)
    .bind(&user_id)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    // 删除使用过的重置令牌
    sqlx::query!("DELETE FROM password_resets WHERE email = ?", email)
        .execute(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    Ok(())
}

// 创建密码重置令牌
pub async fn create_password_reset(pool: &SqlitePool, email: &str) -> Result<String, DbError> {
    // 检查用户是否存在
    let user = sqlx::query!("SELECT id FROM users WHERE email = ?", email)
        .fetch_optional(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    let user_id = match user {
        Some(user) => user.id,
        None => return Err(DbError::NotFound),
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
        .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    // 创建新的重置令牌
    sqlx::query(
        "
        INSERT INTO password_resets (email, token, user_id, created_at, expires_at)
        VALUES (?, ?, ?, ?, ?)
        "
    )
    .bind(email)
    .bind(&token)
    .bind(&user_id)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    Ok(token)
}

// 初始化安全相关的数据库表
pub async fn init_security_tables(pool: &SqlitePool) -> Result<(), DbError> {
    // 创建用户表
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            username TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    // 创建会话表
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    // 创建密码重置表
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS password_resets (
            email TEXT PRIMARY KEY,
            token TEXT NOT NULL,
            user_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    // 创建加密密钥表
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS encryption_keys (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            key_data BLOB NOT NULL,
            nonce BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    Ok(())
}

// 为用户生成加密密钥
pub async fn generate_user_encryption_key(pool: &SqlitePool, user_id: &str) -> Result<(), DbError> {
    // 生成新密钥和nonce
    let key = generate_encryption_key();
    let nonce = generate_nonce();
    
    let id = Uuid::new_v4().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // 存储密钥
    sqlx::query(
        "
        INSERT INTO encryption_keys (id, user_id, key_data, nonce, created_at)
        VALUES (?, ?, ?, ?, ?)
        "
    )
    .bind(&id)
    .bind(user_id)
    .bind(&key.to_vec())
    .bind(&nonce.to_vec())
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    Ok(())
}

// 获取用户的加密密钥
pub async fn get_user_encryption_key(pool: &SqlitePool, user_id: &str) -> Result<([u8; 32], [u8; 12]), DbError> {
    let key_data = sqlx::query!(
        "SELECT key_data, nonce FROM encryption_keys WHERE user_id = ? ORDER BY created_at DESC LIMIT 1",
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    match key_data {
        Some(row) => {
            let key_vec = row.key_data;
            let nonce_vec = row.nonce;
            
            let mut key = [0u8; 32];
            let mut nonce = [0u8; 12];
            
            if key_vec.len() != 32 || nonce_vec.len() != 12 {
                return Err(DbError::InvalidData("Invalid key or nonce length".to_string()));
            }
            
            key.copy_from_slice(&key_vec);
            nonce.copy_from_slice(&nonce_vec);
            
            Ok((key, nonce))
        },
        None => Err(DbError::NotFound),
    }
}
use crate::{DbError, security};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

// 账户相关结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub verification_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub reset_token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub id: String,  // 不是 Option<String>
    pub email: String,
    pub username: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: String,
    pub email: String,
}

// 验证码相关函数

// 生成验证码
pub async fn generate_verification_code(pool: &SqlitePool, email: &str) -> Result<String, DbError> {
    // 生成6位数字验证码
    let code = format!("{:06}", rand::random::<u32>() % 1000000);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let expires_at = now + 10 * 60; // 10分钟过期
    
    // 存储验证码
    sqlx::query(
        "
        INSERT INTO verification_codes (email, code, created_at, expires_at)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(email) DO UPDATE SET
        code = excluded.code,
        created_at = excluded.created_at,
        expires_at = excluded.expires_at
        "
    )
    .bind(email)
    .bind(&code)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    // 在实际应用中，这里应该发送邮件
    // 但在开发阶段，我们直接返回验证码
    Ok(code)
}

// 验证验证码
async fn verify_code(pool: &SqlitePool, email: &str, code: &str) -> Result<bool, DbError> {
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
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    match result {
        Some(row) => Ok(row.code == code),
        None => Ok(false),
    }
}

// 账户相关命令

// 用户注册
#[tauri::command]
pub async fn register_user(
    state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
    request: RegisterRequest,
) -> Result<security::User, String> {
    // 验证验证码
    let is_valid = verify_code(&state.db, &request.email, &request.verification_code)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    if !is_valid {
        return Err("Invalid verification code".to_string());
    }
    
    // 创建用户
    let user = security::create_user(&state.db, &request.email, &request.password)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 为用户生成加密密钥
    security::generate_user_encryption_key(&state.db, &user.id)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 删除已使用的验证码
    let _ = sqlx::query!("DELETE FROM verification_codes WHERE email = ?", request.email)
        .execute(&state.db)
        .await;
    
    Ok(user)
}

// 用户登录
#[tauri::command]
pub async fn login_user(
    state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
    app_handle: tauri::AppHandle,
    request: LoginRequest,
) -> Result<security::Session, String> {
    // 获取设备ID
    let device_id = app_handle.config().identifier.clone();
    
    // 登录用户
    let session = security::login_user(&state.db, &request.email, &request.password, &device_id)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    Ok(session)
}

// 用户注销
#[tauri::command]
pub async fn logout_user(
    state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
    token: String,
) -> Result<(), String> {
    security::logout_session(&state.db, &token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    Ok(())
}

// 获取用户信息
#[tauri::command]
pub async fn get_user_profile(
    state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
    token: String,
) -> Result<UserProfile, String> {
    // 验证会话
    let user = security::verify_session(&state.db, &token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 获取设备数量
    let device_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM sessions WHERE user_id = ?",
        user.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .count;
    
    Ok(UserProfile {
        email: user.email,
        device_count,
        created_at: user.created_at,
    })
}

// 修改密码
#[tauri::command]
pub async fn change_password(
    state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
    token: String,
    request: ChangePasswordRequest,
) -> Result<(), String> {
    // 验证会话
    let user = security::verify_session(&state.db, &token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 修改密码
    security::change_password(&state.db, &user.id, &request.old_password, &request.new_password)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    Ok(())
}

// 请求密码重置
#[tauri::command]
pub async fn request_password_reset(
    state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
    email: String,
) -> Result<(), String> {
    // 创建密码重置令牌
    let token = security::create_password_reset(&state.db, &email)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 在实际应用中，这里应该发送邮件
    // 但在开发阶段，我们只打印令牌
    println!("Password reset token for {}: {}", email, token);
    
    Ok(())
}

// 重置密码
#[tauri::command]
pub async fn reset_password(
    state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
    request: ResetPasswordRequest,
) -> Result<(), String> {
    security::reset_password(&state.db, &request.email, &request.reset_token, &request.new_password)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    Ok(())
}

// 更新用户个人信息
#[tauri::command]
pub async fn update_user_profile(
    state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
    token: String,
    request: UpdateProfileRequest,
) -> Result<UserProfile, String> {
    // 验证会话
    let user = security::verify_session(&state.db, &token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 更新用户信息
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // 更新用户邮箱
    sqlx::query(
        "
        UPDATE users SET
        email = ?,
        updated_at = ?
        WHERE id = ?
        "
    )
    .bind(&request.email)
    .bind(now)
    .bind(&user.id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Database error: {}", e))?;
    
    // 获取设备数量
    let device_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM sessions WHERE user_id = ?",
        user.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .count;
    
    Ok(UserProfile {
        email: request.email,
        device_count,
        created_at: user.created_at,
    })
}

// 初始化账户相关的数据库表
pub async fn init_account_tables(pool: &SqlitePool) -> Result<(), DbError> {
    // 创建验证码表
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS verification_codes (
            email TEXT PRIMARY KEY,
            code TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL
        )
        "
    )
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    Ok(())
}
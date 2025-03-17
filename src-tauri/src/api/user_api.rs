use tauri::{State, AppHandle};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::service::auth_service::AuthService;
use crate::service::user_service::UserService;
use crate::entity::session::Session;
use crate::entity::user::UserProfile;

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
    pub token: String,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub reset_token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub token: String,
    pub username: String,
    pub email: String,
}

#[tauri::command]
pub async fn register_user(
    state: State<'_, Arc<AppState>>,
    request: RegisterRequest,
) -> Result<UserProfile, String> {
    // 注册用户
    let user = UserService::register(
        &state.db, 
        &request.email, 
        &request.password, 
        &request.verification_code
    )
    .await
    .map_err(|e| format!("{:?}", e))?;
    
    // 返回用户资料
    Ok(UserProfile {
        id: user.id,
        email: user.email,
        username: user.username,
        created_at: user.created_at,
        updated_at: user.updated_at,
    })
}

#[tauri::command]
pub async fn login_user(
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
    request: LoginRequest,
) -> Result<Session, String> {
    // 获取设备ID
    let device_id = app_handle.config().identifier.clone();
    
    // 登录用户
    AuthService::login(&state.db, &request.email, &request.password, &device_id)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn logout_user(
    state: State<'_, Arc<AppState>>,
    token: String,
) -> Result<(), String> {
    // 注销用户
    AuthService::logout(&state.db, &token)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn get_user_profile(
    state: State<'_, Arc<AppState>>,
    token: String,
) -> Result<UserProfile, String> {
    // 验证会话
    let user = AuthService::verify_session(&state.db, &token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 获取用户资料
    UserService::get_profile(&state.db, &user.id)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn update_user_profile(
    state: State<'_, Arc<AppState>>,
    request: UpdateProfileRequest,
) -> Result<UserProfile, String> {
    // 验证会话
    let user = AuthService::verify_session(&state.db, &request.token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 更新用户资料
    UserService::update_profile(&state.db, &user.id, &request.username, &request.email)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn change_password(
    state: State<'_, Arc<AppState>>,
    request: ChangePasswordRequest,
) -> Result<(), String> {
    // 验证会话
    let user = AuthService::verify_session(&state.db, &request.token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 修改密码
    AuthService::change_password(&state.db, &user.id, &request.old_password, &request.new_password)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn request_password_reset(
    state: State<'_, Arc<AppState>>,
    email: String,
) -> Result<(), String> {
    // 创建密码重置令牌
    let token = AuthService::request_password_reset(&state.db, &email)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 在实际应用中，这里应该发送邮件
    // 但在开发阶段，我们只打印令牌
    println!("密码重置令牌 ({}): {}", email, token);
    
    Ok(())
}

#[tauri::command]
pub async fn reset_password(
    state: State<'_, Arc<AppState>>,
    request: ResetPasswordRequest,
) -> Result<(), String> {
    AuthService::reset_password(&state.db, &request.email, &request.reset_token, &request.new_password)
        .await
        .map_err(|e| format!("{:?}", e))
}
use tauri::{State, AppHandle};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::service::clipboard_service::ClipboardService;
use crate::service::auth_service::AuthService;
use crate::entity::clipboard_item::{ClipboardItem, ClipboardItemRequest, ClipboardItemUpdateRequest};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetClipboardItemsRequest {
    pub token: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddClipboardItemRequest {
    pub token: String,
    pub content: String,
    pub content_type: String,
    pub encrypt: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClipboardItemRequest {
    pub token: String,
    pub id: String,
    pub content: String,
    pub content_type: String,
    pub encrypt: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteClipboardItemRequest {
    pub token: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchClipboardItemsRequest {
    pub token: String,
    pub query: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub async fn get_clipboard_items(
    state: State<'_, Arc<AppState>>,
    request: GetClipboardItemsRequest,
) -> Result<Vec<ClipboardItem>, String> {
    // 验证会话
    let user = AuthService::verify_session(&state.db, &request.token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 获取剪贴板项目
    let limit = request.limit.unwrap_or(50);
    let offset = request.offset.unwrap_or(0);
    
    ClipboardService::get_items(&state.db, &user.id, limit, offset)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn add_clipboard_item(
    state: State<'_, Arc<AppState>>,
    request: AddClipboardItemRequest,
) -> Result<ClipboardItem, String> {
    // 验证会话
    let user = AuthService::verify_session(&state.db, &request.token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 创建请求对象
    let item_request = ClipboardItemRequest {
        content: request.content,
        content_type: request.content_type,
        encrypt: request.encrypt,
    };
    
    // 添加剪贴板项目
    ClipboardService::add_item(&state.db, &user.id, &item_request)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn update_clipboard_item(
    state: State<'_, Arc<AppState>>,
    request: UpdateClipboardItemRequest,
) -> Result<ClipboardItem, String> {
    // 验证会话
    let user = AuthService::verify_session(&state.db, &request.token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 创建请求对象
    let item_request = ClipboardItemUpdateRequest {
        id: request.id,
        content: request.content,
        content_type: request.content_type,
        encrypt: request.encrypt,
    };
    
    // 更新剪贴板项目
    ClipboardService::update_item(&state.db, &user.id, &item_request)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn delete_clipboard_item(
    state: State<'_, Arc<AppState>>,
    request: DeleteClipboardItemRequest,
) -> Result<(), String> {
    // 验证会话
    let user = AuthService::verify_session(&state.db, &request.token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 删除剪贴板项目
    ClipboardService::delete_item(&state.db, &user.id, &request.id)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn search_clipboard_items(
    state: State<'_, Arc<AppState>>,
    request: SearchClipboardItemsRequest,
) -> Result<Vec<ClipboardItem>, String> {
    // 验证会话
    let user = AuthService::verify_session(&state.db, &request.token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 搜索剪贴板项目
    let limit = request.limit.unwrap_or(50);
    let offset = request.offset.unwrap_or(0);
    
    ClipboardService::search_items(&state.db, &user.id, &request.query, limit, offset)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn start_clipboard_monitor(
    state: State<'_, Arc<AppState>>,
    app_handle: AppHandle,
    token: String,
) -> Result<(), String> {
    // 验证会话
    let user = AuthService::verify_session(&state.db, &token)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 启动剪贴板监控
    let db = state.db.clone();
    let user_id = user.id.clone();
    
    // 创建一个新线程来监控剪贴板变化
    tauri::async_runtime::spawn(async move {
        let mut last_content = String::new();
        
        loop {
            // 使用 tauri_plugin_clipboard_manager 获取剪贴板内容
            if let Ok(content) = app_handle.clipboard().read_text() {
                if !content.is_empty() && content != last_content {
                    // 内容变化，保存到数据库
                    let item_request = ClipboardItemRequest {
                        content: content.clone(),
                        content_type: "text/plain".to_string(),
                        encrypt: false, // 默认不加密
                    };
                    
                    if let Err(e) = ClipboardService::add_item(&db, &user_id, &item_request).await {
                        eprintln!("保存剪贴板内容失败: {:?}", e);
                    }
                    
                    last_content = content;
                }
            }
            
            // 等待一段时间再检查
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    });
    
    Ok(())
}
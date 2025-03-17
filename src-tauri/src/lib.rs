use sqlx::SqlitePool;
use std::sync::Arc;

// 导入模块
pub mod entity;
pub mod repository;
pub mod service;
pub mod api;
pub mod error;
pub mod util;

// 应用状态
pub struct AppState {
    pub db: SqlitePool,
    pub cache_queue: Arc<tokio::sync::Mutex<Vec<String>>>, // 简化示例
}

// 初始化数据库
async fn init_database() -> Result<SqlitePool, error::AppError> {
    // 数据库初始化代码...
    // 这里是简化的示例
    let pool = SqlitePool::connect("sqlite:sharing-copyboard.db")
        .await
        .map_err(|e| error::AppError::DatabaseError(e.to_string()))?;
    
    // 初始化表
    // ...
    
    Ok(pool)
}

// 简单的问候函数，用于测试
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 应用入口
pub fn run() {
    tauri::async_runtime::block_on(async {
        // 初始化数据库
        let db = match init_database().await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("数据库初始化失败: {:?}", e);
                return;
            }
        };
        
        // 初始化缓存系统 - 直接创建而不是使用不存在的模块
        let cache_queue = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        
        // 创建应用状态
        let app_state = Arc::new(AppState {
            db,
            cache_queue,
        });
        
        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_clipboard_manager::init())
            .manage(app_state)
            .invoke_handler(tauri::generate_handler![
                greet, 
                // 剪贴板相关命令
                api::clipboard_api::get_clipboard_items,
                api::clipboard_api::add_clipboard_item,
                api::clipboard_api::update_clipboard_item,
                api::clipboard_api::delete_clipboard_item,
                api::clipboard_api::search_clipboard_items,
                api::clipboard_api::start_clipboard_monitor,
                
                // 账户相关命令
                api::user_api::register_user,
                api::user_api::login_user,
                api::user_api::logout_user,
                api::user_api::get_user_profile,
                api::user_api::update_user_profile,
                api::user_api::change_password,
                api::user_api::request_password_reset,
                api::user_api::reset_password
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}

use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri::Emitter;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, Row};
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::sync::Mutex;

// 导入新模块
mod sync;
mod security;
mod account;

// 全局状态结构体
pub struct AppState {
    pub db: SqlitePool,
    pub cache_queue: Mutex<VecDeque<ClipboardItem>>,
}

// 剪贴板项目数据模型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipboardItem {
    pub id: String,
    pub content: String,
    pub title: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_pinned: bool,
}

// 数据库错误类型
#[derive(Debug, Serialize)]
pub enum DbError {
    ConnectionError(String),
    QueryError(String),
    NotFound,
    InvalidData(String),
}

impl From<sqlx::Error> for DbError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => DbError::NotFound,
            _ => DbError::QueryError(error.to_string()),
        }
    }
}

// 初始化数据库连接
async fn init_database() -> Result<SqlitePool, DbError> {
    let db_path = "sqlite:sharing-copyboard.db";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_path)
        .await
        .map_err(|e| DbError::ConnectionError(e.to_string()))?;
    
    // 创建剪贴板项目表
    sqlx::query("
        CREATE TABLE IF NOT EXISTS clipboard_items (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            title TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            is_pinned INTEGER DEFAULT 0
        )
    ")
    .execute(&pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    // 创建同步状态表
    sqlx::query("
        CREATE TABLE IF NOT EXISTS sync_status (
            item_id TEXT PRIMARY KEY,
            is_synced INTEGER DEFAULT 0,
            last_sync_attempt INTEGER,
            FOREIGN KEY (item_id) REFERENCES clipboard_items(id) ON DELETE CASCADE
        )
    ")
    .execute(&pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    // 创建用户设置表
    sqlx::query("
        CREATE TABLE IF NOT EXISTS user_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        )
    ")
    .execute(&pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;
    
    // 初始化安全相关的数据库表
    security::init_security_tables(&pool).await
        .map_err(|e| DbError::QueryError(format!("Failed to init security tables: {:?}", e)))?;
    
    // 初始化账户相关的数据库表
    account::init_account_tables(&pool).await
        .map_err(|e| DbError::QueryError(format!("Failed to init account tables: {:?}", e)))?;
    
    Ok(pool)
}

// 剪贴板监听命令
#[tauri::command]
async fn start_clipboard_monitor(app: tauri::AppHandle) {
    let mut last_content = String::new();
    loop {
        let content = app.clipboard().read_text().unwrap_or_default();
        if !content.is_empty() && content != last_content {
            app.emit("clipboard_update", content.clone()).unwrap();
            last_content = content;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

// 问候命令
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 剪贴板数据访问层实现
pub mod clipboard_dao {
    use super::*;
    use uuid::Uuid;
    use std::time::{SystemTime, UNIX_EPOCH};

    // 获取当前时间戳
    fn get_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    // 添加剪贴板项目
    pub async fn add_item(pool: &SqlitePool, content: String, title: Option<String>) -> Result<ClipboardItem, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = get_timestamp();
        let is_pinned = false;

        // 检查内容是否为空
        if content.trim().is_empty() {
            return Err(DbError::InvalidData("Content cannot be empty".to_string()));
        }

        // 插入数据
        sqlx::query(
            "
            INSERT INTO clipboard_items (id, content, title, created_at, updated_at, is_pinned)
            VALUES (?, ?, ?, ?, ?, ?)
            "
        )
        .bind(&id)
        .bind(&content)
        .bind(&title)
        .bind(now)
        .bind(now)
        .bind(is_pinned)
        .execute(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        // 同时创建同步状态记录
        sqlx::query(
            "
            INSERT INTO sync_status (item_id, is_synced, last_sync_attempt)
            VALUES (?, 0, NULL)
            "
        )
        .bind(&id)
        .execute(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        Ok(ClipboardItem {
            id,
            content,
            title,
            created_at: now,
            updated_at: now,
            is_pinned,
        })
    }

    // 获取剪贴板项目
    pub async fn get_item(pool: &SqlitePool, id: &str) -> Result<ClipboardItem, DbError> {
        let item = sqlx::query(
            "
            SELECT id, content, title, created_at, updated_at, is_pinned
            FROM clipboard_items
            WHERE id = ?
            "
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.into())?;

        Ok(ClipboardItem {
            id: item.get("id"),
            content: item.get("content"),
            title: item.get("title"),
            created_at: item.get("created_at"),
            updated_at: item.get("updated_at"),
            is_pinned: item.get::<i64, _>("is_pinned") != 0,
        })
    }

    // 获取所有剪贴板项目
    pub async fn get_all_items(pool: &SqlitePool, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ClipboardItem>, DbError> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let items = sqlx::query(
            "
            SELECT id, content, title, created_at, updated_at, is_pinned
            FROM clipboard_items
            ORDER BY is_pinned DESC, updated_at DESC
            LIMIT ? OFFSET ?
            "
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        let mut result = Vec::with_capacity(items.len());
        for item in items {
            result.push(ClipboardItem {
                id: item.get("id"),
                content: item.get("content"),
                title: item.get("title"),
                created_at: item.get("created_at"),
                updated_at: item.get("updated_at"),
                is_pinned: item.get::<i64, _>("is_pinned") != 0,
            });
        }

        Ok(result)
    }

    // 更新剪贴板项目
    pub async fn update_item(pool: &SqlitePool, id: &str, content: Option<String>, title: Option<String>, is_pinned: Option<bool>) -> Result<ClipboardItem, DbError> {
        // 先检查项目是否存在
        let item = get_item(pool, id).await?;
        let now = get_timestamp();

        // 构建更新SQL
        let mut query = String::from("UPDATE clipboard_items SET updated_at = ?");
        let mut params = Vec::new();
        params.push(now.to_string());

        if let Some(content) = &content {
            if content.trim().is_empty() {
                return Err(DbError::InvalidData("Content cannot be empty".to_string()));
            }
            query.push_str(", content = ?");
            params.push(content.clone());
        }

        if title.is_some() {
            query.push_str(", title = ?");
            params.push(title.clone().unwrap_or_default());
        }

        if let Some(is_pinned) = is_pinned {
            query.push_str(", is_pinned = ?");
            params.push(if is_pinned { "1".to_string() } else { "0".to_string() });
        }

        query.push_str(" WHERE id = ?");
        params.push(id.to_string());

        // 执行更新
        let mut db_query = sqlx::query(&query);
        for param in params {
            db_query = db_query.bind(param);
        }

        db_query
            .execute(pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        // 更新同步状态
        sqlx::query(
            "
            UPDATE sync_status
            SET is_synced = 0
            WHERE item_id = ?
            "
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        // 返回更新后的项目
        get_item(pool, id).await
    }

    // 删除剪贴板项目
    pub async fn delete_item(pool: &SqlitePool, id: &str) -> Result<(), DbError> {
        // 先检查项目是否存在
        let _ = get_item(pool, id).await?;

        // 删除项目
        sqlx::query("DELETE FROM clipboard_items WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

        // 同步状态表会通过外键级联删除

        Ok(())
    }

    // 搜索剪贴板项目
    pub async fn search_items(pool: &SqlitePool, query: &str, limit: Option<i64>) -> Result<Vec<ClipboardItem>, DbError> {
        let limit = limit.unwrap_or(100);
        let search_query = format!("%{}%", query);

        let items = sqlx::query(
            "
            SELECT id, content, title, created_at, updated_at, is_pinned
            FROM clipboard_items
            WHERE content LIKE ? OR title LIKE ?
            ORDER BY is_pinned DESC, updated_at DESC
            LIMIT ?
            "
        )
        .bind(&search_query)
        .bind(&search_query)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

        let mut result = Vec::with_capacity(items.len());
        for item in items {
            result.push(ClipboardItem {
                id: item.get("id"),
                content: item.get("content"),
                title: item.get("title"),
                created_at: item.get("created_at"),
                updated_at: item.get("updated_at"),
                is_pinned: item.get::<i64, _>("is_pinned") != 0,
            });
        }

        Ok(result)
    }
}

// 离线缓存系统实现
pub mod cache_system {
    use super::*;
    use std::sync::Arc;

    const MAX_CACHE_SIZE: usize = 100;

    // 初始化缓存队列
    pub fn init_cache() -> Mutex<VecDeque<ClipboardItem>> {
        Mutex::new(VecDeque::with_capacity(MAX_CACHE_SIZE))
    }

    // 添加项目到缓存
    pub fn add_to_cache(cache: &Mutex<VecDeque<ClipboardItem>>, item: ClipboardItem) {
        let mut cache = cache.lock().unwrap();
        
        // 检查是否已存在相同ID的项目
        if let Some(pos) = cache.iter().position(|x| x.id == item.id) {
            cache.remove(pos);
        }
        
        // 如果缓存已满，移除最旧的项目
        if cache.len() >= MAX_CACHE_SIZE {
            cache.pop_back();
        }
        
        // 添加到队列前端
        cache.push_front(item);
    }

    // 从缓存获取项目
    pub fn get_from_cache(cache: &Mutex<VecDeque<ClipboardItem>>, id: &str) -> Option<ClipboardItem> {
        let cache = cache.lock().unwrap();
        cache.iter().find(|item| item.id == id).cloned()
    }

    // 从缓存移除项目
    pub fn remove_from_cache(cache: &Mutex<VecDeque<ClipboardItem>>, id: &str) {
        let mut cache = cache.lock().unwrap();
        if let Some(pos) = cache.iter().position(|x| x.id == id) {
            cache.remove(pos);
        }
    }

    // 获取所有缓存项目
    pub fn get_all_cached(cache: &Mutex<VecDeque<ClipboardItem>>) -> Vec<ClipboardItem> {
        let cache = cache.lock().unwrap();
        cache.iter().cloned().collect()
    }

    // 清空缓存
    pub fn clear_cache(cache: &Mutex<VecDeque<ClipboardItem>>) {
        let mut cache = cache.lock().unwrap();
        cache.clear();
    }
}

// 剪贴板相关的Tauri命令
#[tauri::command]
async fn get_clipboard_items(state: tauri::State<'_, Arc<AppState>>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ClipboardItem>, String> {
    clipboard_dao::get_all_items(&state.db, limit, offset)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
async fn add_clipboard_item(state: tauri::State<'_, Arc<AppState>>, content: String, title: Option<String>) -> Result<ClipboardItem, String> {
    let item = clipboard_dao::add_item(&state.db, content, title)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 添加到缓存
    cache_system::add_to_cache(&state.cache_queue, item.clone());
    
    Ok(item)
}

#[tauri::command]
async fn update_clipboard_item(state: tauri::State<'_, Arc<AppState>>, id: String, content: Option<String>, title: Option<String>, is_pinned: Option<bool>) -> Result<ClipboardItem, String> {
    let item = clipboard_dao::update_item(&state.db, &id, content, title, is_pinned)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 更新缓存
    cache_system::add_to_cache(&state.cache_queue, item.clone());
    
    Ok(item)
}

#[tauri::command]
async fn delete_clipboard_item(state: tauri::State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    clipboard_dao::delete_item(&state.db, &id)
        .await
        .map_err(|e| format!("{:?}", e))?;
    
    // 从缓存中移除
    cache_system::remove_from_cache(&state.cache_queue, &id);
    
    Ok(())
}

#[tauri::command]
async fn search_clipboard_items(state: tauri::State<'_, Arc<AppState>>, query: String, limit: Option<i64>) -> Result<Vec<ClipboardItem>, String> {
    clipboard_dao::search_items(&state.db, &query, limit)
        .await
        .map_err(|e| format!("{:?}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
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
        
        // 初始化缓存系统
        let cache_queue = cache_system::init_cache();
        
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
                start_clipboard_monitor,
                get_clipboard_items,
                add_clipboard_item,
                update_clipboard_item,
                delete_clipboard_item,
                search_clipboard_items,
                // 账户相关命令
                account::register_user,
                account::login_user,
                account::logout_user,
                account::get_user_profile,
                account::update_user_profile,
                account::change_password,
                account::request_password_reset,
                account::reset_password
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}

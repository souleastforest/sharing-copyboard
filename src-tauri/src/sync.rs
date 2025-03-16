use crate::{AppState, ClipboardItem, DbError};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, Row};  // 添加 Row trait 导入
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;
use tokio::net::TcpStream;
use tokio::sync::Mutex as TokioMutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};
use uuid::Uuid;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

// 设备信息结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub last_sync: i64,
}

// 同步消息结构体
#[derive(Debug, Serialize, Deserialize)]
pub enum SyncMessage {
    Connect {
        device_id: String,
        device_name: String,
    },
    ItemUpdate(ClipboardItem),
    ItemDelete {
        id: String,
    },
    SyncRequest {
        since_timestamp: i64,
    },
    SyncResponse {
        items: Vec<ClipboardItem>,
    },
    Error {
        code: String,
        message: String,
    },
}

// WebSocket连接管理器
pub struct WebSocketManager {
    ws_stream: TokioMutex<Option<WebSocketStream<TcpStream>>>,
    device_id: String,
    device_name: String,
    server_url: String,
    connected: TokioMutex<bool>,
    reconnect_attempts: TokioMutex<u32>,
}

impl WebSocketManager {
    // 创建新的WebSocket管理器
    pub fn new(device_id: String, device_name: String, server_url: String) -> Self {
        Self {
            ws_stream: TokioMutex::new(None),
            device_id,
            device_name,
            server_url,
            connected: TokioMutex::new(false),
            reconnect_attempts: TokioMutex::new(0),
        }
    }

    // 连接到WebSocket服务器
    pub async fn connect(&self) -> Result<(), String> {
        let mut connected = self.connected.lock().await;
        if *connected {
            return Ok(());
        }

        let url = url::Url::parse(&self.server_url)
            .map_err(|e| format!("Invalid WebSocket URL: {}", e))?;

        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                let mut stream_lock = self.ws_stream.lock().await;
                *stream_lock = Some(ws_stream);
                *connected = true;
                *self.reconnect_attempts.lock().await = 0;
                
                // 发送连接消息
                self.send_message(SyncMessage::Connect {
                    device_id: self.device_id.clone(),
                    device_name: self.device_name.clone(),
                }).await?
            }
            Err(e) => {
                let mut attempts = self.reconnect_attempts.lock().await;
                *attempts += 1;
                return Err(format!("Failed to connect: {} (attempt {})", e, *attempts));
            }
        }

        Ok(())
    }

    // 发送消息
    pub async fn send_message(&self, message: SyncMessage) -> Result<(), String> {
        let json = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        let mut stream_lock = self.ws_stream.lock().await;
        if let Some(stream) = &mut *stream_lock {
            stream
                .send(Message::Text(json))
                .await
                .map_err(|e| format!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err("Not connected to WebSocket server".to_string())
        }
    }

    // 接收消息处理循环
    pub async fn start_message_loop(
        self: Arc<Self>,
        app_state: Arc<AppState>,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            // 确保连接
            if !*self.connected.lock().await {
                if let Err(e) = self.connect().await {
                    eprintln!("Connection error: {}", e);
                    // 指数退避重连
                    let attempts = *self.reconnect_attempts.lock().await;
                    let delay = std::cmp::min(2u64.pow(attempts), 60) * 1000;
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    continue;
                }
            }

            tokio::select! {
                _ = interval.tick() => {
                    // 发送心跳或同步请求
                    if *self.connected.lock().await {
                        let last_sync = get_last_sync_timestamp(&app_state.db).await
                            .unwrap_or(0);
                        
                        if let Err(e) = self.send_message(SyncMessage::SyncRequest {
                            since_timestamp: last_sync,
                        }).await {
                            eprintln!("Failed to send sync request: {}", e);
                            *self.connected.lock().await = false;
                        }
                    }
                }
                
                msg = async {
                    let mut stream_lock = self.ws_stream.lock().await;
                    if let Some(stream) = &mut *stream_lock {
                        stream.next().await
                    } else {
                        None
                    }
                } => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            match serde_json::from_str::<SyncMessage>(&text) {
                                Ok(sync_msg) => {
                                    self.handle_message(sync_msg, app_state.clone(), app_handle.clone()).await;
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse message: {}", e);
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            *self.connected.lock().await = false;
                            eprintln!("WebSocket connection closed");
                        }
                        Some(Err(e)) => {
                            *self.connected.lock().await = false;
                            eprintln!("WebSocket error: {}", e);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // 处理接收到的消息
    async fn handle_message(
        &self,
        message: SyncMessage,
        app_state: Arc<AppState>,
        app_handle: tauri::AppHandle,
    ) {
        match message {
            SyncMessage::ItemUpdate(item) => {
                // 处理项目更新
                match sync_item_from_remote(&app_state.db, item.clone()).await {
                    Ok(_) => {
                        // 更新缓存
                        crate::cache_system::add_to_cache(&app_state.cache_queue, item.clone());
                        
                        // 通知前端
                        let _ = app_handle.emit("remote_item_update", item);
                    }
                    Err(e) => {
                        eprintln!("Failed to sync remote item: {:?}", e);
                    }
                }
            }
            SyncMessage::ItemDelete { id } => {
                // 处理项目删除
                match delete_synced_item(&app_state.db, &id).await {
                    Ok(_) => {
                        // 从缓存中移除
                        crate::cache_system::remove_from_cache(&app_state.cache_queue, &id);
                        
                        // 通知前端
                        let _ = app_handle.emit("remote_item_delete", id);
                    }
                    Err(e) => {
                        eprintln!("Failed to delete synced item: {:?}", e);
                    }
                }
            }
            SyncMessage::SyncResponse { items } => {
                // 处理同步响应
                for item in items {
                    if let Err(e) = sync_item_from_remote(&app_state.db, item.clone()).await {
                        eprintln!("Failed to sync item: {:?}", e);
                    } else {
                        // 更新缓存
                        crate::cache_system::add_to_cache(&app_state.cache_queue, item.clone());
                    }
                }
                
                // 更新最后同步时间
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                
                let _ = update_last_sync_timestamp(&app_state.db, now).await;
                
                // 通知前端刷新
                let _ = app_handle.emit("sync_completed", ());
            }
            SyncMessage::Error { code, message } => {
                eprintln!("Received error from server: {} - {}", code, message);
                // 通知前端显示错误
                let _ = app_handle.emit("sync_error", format!("{}: {}", code, message));
            }
            _ => {}
        }
    }

    // 断开连接
    pub async fn disconnect(&self) -> Result<(), String> {
        let mut connected = self.connected.lock().await;
        if !*connected {
            return Ok(());
        }

        let mut stream_lock = self.ws_stream.lock().await;
        if let Some(stream) = &mut *stream_lock {
            stream
                .close(None)
                .await
                .map_err(|e| format!("Failed to close connection: {}", e))?;
        }

        *stream_lock = None;
        *connected = false;
        Ok(())
    }
}

// 数据同步相关的数据库操作

// 获取最后同步时间戳
async fn get_last_sync_timestamp(pool: &SqlitePool) -> Result<i64, DbError> {
    let result = sqlx::query!("SELECT value FROM user_settings WHERE key = 'last_sync_timestamp'")
        .fetch_optional(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

    match result {
        Some(row) => Ok(row.value.parse::<i64>().unwrap_or(0)),
        None => Ok(0),
    }
}

// 更新最后同步时间戳
async fn update_last_sync_timestamp(pool: &SqlitePool, timestamp: i64) -> Result<(), DbError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query(
        "
        INSERT INTO user_settings (key, value, updated_at)
        VALUES ('last_sync_timestamp', ?, ?)
        ON CONFLICT(key) DO UPDATE SET
        value = excluded.value,
        updated_at = excluded.updated_at
        "
    )
    .bind(timestamp.to_string())
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;

    Ok(())
}

// 从远程同步项目
async fn sync_item_from_remote(pool: &SqlitePool, item: ClipboardItem) -> Result<(), DbError> {
    // 检查项目是否已存在
    let existing = sqlx::query!("SELECT id, updated_at FROM clipboard_items WHERE id = ?", item.id)
        .fetch_optional(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

    match existing {
        Some(row) => {
            // 如果远程项目更新时间更新，则更新本地项目
            if item.updated_at > row.updated_at {
                sqlx::query(
                    "
                    UPDATE clipboard_items SET
                    content = ?,
                    title = ?,
                    updated_at = ?,
                    is_pinned = ?
                    WHERE id = ?
                    "
                )
                .bind(&item.content)
                .bind(&item.title)
                .bind(item.updated_at)
                .bind(item.is_pinned as i64)
                .bind(&item.id)
                .execute(pool)
                .await
                .map_err(|e| DbError::QueryError(e.to_string()))?;

                // 更新同步状态
                sqlx::query(
                    "
                    UPDATE sync_status SET
                    is_synced = 1,
                    last_sync_attempt = ?
                    WHERE item_id = ?
                    "
                )
                .bind(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64)
                .bind(&item.id)
                .execute(pool)
                .await
                .map_err(|e| DbError::QueryError(e.to_string()))?;
            }
        }
        None => {
            // 如果项目不存在，则插入新项目
            sqlx::query(
                "
                INSERT INTO clipboard_items (id, content, title, created_at, updated_at, is_pinned)
                VALUES (?, ?, ?, ?, ?, ?)
                "
            )
            .bind(&item.id)
            .bind(&item.content)
            .bind(&item.title)
            .bind(item.created_at)
            .bind(item.updated_at)
            .bind(item.is_pinned as i64)
            .execute(pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;

            // 创建同步状态记录
            sqlx::query(
                "
                INSERT INTO sync_status (item_id, is_synced, last_sync_attempt)
                VALUES (?, 1, ?)
                "
            )
            .bind(&item.id)
            .bind(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64)
            .execute(pool)
            .await
            .map_err(|e| DbError::QueryError(e.to_string()))?;
        }
    }

    Ok(())
}

// 删除同步项目
async fn delete_synced_item(pool: &SqlitePool, id: &str) -> Result<(), DbError> {
    // 检查项目是否存在
    let exists = sqlx::query!("SELECT id FROM clipboard_items WHERE id = ?", id)
        .fetch_optional(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

    if exists.is_none() {
        return Ok(());
    }

    // 删除项目
    sqlx::query("DELETE FROM clipboard_items WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

    // 同步状态表会通过外键级联删除

    Ok(())
}

// 获取未同步的项目
pub async fn get_unsynced_items(pool: &SqlitePool, limit: Option<i64>) -> Result<Vec<ClipboardItem>, DbError> {
    let limit = limit.unwrap_or(50);

    let items = sqlx::query(
        "
        SELECT c.id, c.content, c.title, c.created_at, c.updated_at, c.is_pinned
        FROM clipboard_items c
        JOIN sync_status s ON c.id = s.item_id
        WHERE s.is_synced = 0
        LIMIT ?
        "
    )
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

// 标记项目为已同步
pub async fn mark_item_synced(pool: &SqlitePool, id: &str) -> Result<(), DbError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query(
        "
        UPDATE sync_status SET
        is_synced = 1,
        last_sync_attempt = ?
        WHERE item_id = ?
        "
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;

    Ok(())
}

// 设备管理功能

// 获取已绑定设备列表
pub async fn get_bound_devices(pool: &SqlitePool) -> Result<Vec<DeviceInfo>, DbError> {
    let devices = sqlx::query(
        "
        SELECT value FROM user_settings WHERE key = 'bound_devices'
        "
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;

    match devices {
        Some(row) => {
            let devices_json: String = row.get("value");
            let devices: Vec<DeviceInfo> = serde_json::from_str(&devices_json)
                .map_err(|e| DbError::QueryError(format!("Failed to parse devices: {}", e)))?;
            Ok(devices)
        }
        None => Ok(Vec::new()),
    }
}

// 添加绑定设备
pub async fn add_bound_device(pool: &SqlitePool, device: DeviceInfo) -> Result<(), DbError> {
    // 获取当前设备列表
    let mut devices = get_bound_devices(pool).await?;

    // 检查设备数量限制（最多5个设备）
    if devices.len() >= 5 {
        return Err(DbError::InvalidData("Maximum number of devices (5) reached".to_string()));
    }

    // 检查设备是否已存在
    if let Some(pos) = devices.iter().position(|d| d.device_id == device.device_id) {
        // 更新现有设备
        devices[pos] = device;
    } else {
        // 添加新设备
        devices.push(device);
    }

    // 保存设备列表
    let devices_json = serde_json::to_string(&devices)
        .map_err(|e| DbError::QueryError(format!("Failed to serialize devices: {}", e)))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query(
        "
        INSERT INTO user_settings (key, value, updated_at)
        VALUES ('bound_devices', ?, ?)
        ON CONFLICT(key) DO UPDATE SET
        value = excluded.value,
        updated_at = excluded.updated_at
        "
    )
    .bind(devices_json)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;

    Ok(())
}

// 移除绑定设备
pub async fn remove_bound_device(pool: &SqlitePool, device_id: &str) -> Result<(), DbError> {
    // 获取当前设备列表
    let mut devices = get_bound_devices(pool).await?;

    // 移除设备
    devices.retain(|d| d.device_id != device_id);

    // 保存设备列表
    let devices_json = serde_json::to_string(&devices)
        .map_err(|e| DbError::QueryError(format!("Failed to serialize devices: {}", e)))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query(
        "
        INSERT INTO user_settings (key, value, updated_at)
        VALUES ('bound_devices', ?, ?)
        ON CONFLICT(key) DO UPDATE SET
        value = excluded.value,
        updated_at = excluded.updated_at
        "
    )
    .bind(devices_json)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| DbError::QueryError(e.to_string()))?;

    Ok(())
}
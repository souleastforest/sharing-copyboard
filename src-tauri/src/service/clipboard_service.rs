use sqlx::SqlitePool;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::entity::clipboard_item::{ClipboardItem, ClipboardItemRequest, ClipboardItemUpdateRequest};
use crate::repository::clipboard_repository::ClipboardRepository;
use crate::error::AppError;
use crate::util::crypto;
use crate::repository::encryption_repository::EncryptionRepository;

pub struct ClipboardService;

impl ClipboardService {
    pub async fn get_items(
        pool: &SqlitePool, 
        user_id: &str, 
        limit: i64, 
        offset: i64
    ) -> Result<Vec<ClipboardItem>, AppError> {
        ClipboardRepository::find_all_by_user_id(pool, user_id, limit, offset).await
    }
    
    pub async fn add_item(
        pool: &SqlitePool, 
        user_id: &str, 
        request: &ClipboardItemRequest
    ) -> Result<ClipboardItem, AppError> {
        // let id = Uuid::new_v4().to_string();
        // let now = SystemTime::now()
        //     .duration_since(UNIX_EPOCH)
        //     .unwrap()
        //     .as_secs() as i64;
        
        let mut content = request.content.clone();
        let mut encrypted = false;
        
        // 如果需要加密
        if request.encrypt {
            // 获取用户的加密密钥
            let encryption_key = EncryptionRepository::find_by_user_id(pool, user_id).await?
                .ok_or_else(|| AppError::NotFound("加密密钥不存在".to_string()))?;
            
            // 加密内容
            let nonce = crypto::generate_nonce();
            let encrypted_data = crypto::encrypt_data(
                content.as_bytes(),
                &encryption_key.key_data,
                &nonce
            ).map_err(|e| AppError::CryptoError(e))?;
            
            // 将加密后的数据和nonce一起存储
            let combined = [&nonce[..], &encrypted_data[..]].concat();
            content = base64::encode(combined);
            encrypted = true;
        }
        
        let item = ClipboardItem::new(user_id, &content, &request.content_type.clone(), encrypted);
        
        ClipboardRepository::save(pool, &item).await?;
        
        Ok(item)
    }
    
    pub async fn update_item(
        pool: &SqlitePool, 
        user_id: &str, 
        request: &ClipboardItemUpdateRequest
    ) -> Result<ClipboardItem, AppError> {
        // 检查项目是否存在
        let existing = ClipboardRepository::find_by_id(pool, &request.id, user_id).await?
            .ok_or_else(|| AppError::NotFound("剪贴板项目不存在".to_string()))?;
        
        let mut content = request.content.clone();
        let mut encrypted = false;
        
        // 如果需要加密
        if request.encrypt {
            // 获取用户的加密密钥
            let encryption_key = EncryptionRepository::find_by_user_id(pool, user_id).await?
                .ok_or_else(|| AppError::NotFound("加密密钥不存在".to_string()))?;
            
            // 加密内容
            let nonce = crypto::generate_nonce();
            let encrypted_data = crypto::encrypt_data(
                content.as_bytes(),
                &encryption_key.key_data,
                &nonce
            ).map_err(|e| AppError::CryptoError(e))?;
            
            // 将加密后的数据和nonce一起存储
            let combined = [&nonce[..], &encrypted_data[..]].concat();
            content = base64::encode(combined);
            encrypted = true;
        }
        let item = ClipboardItem::new(user_id, &content, &request.content_type.clone(), encrypted);
        
        ClipboardRepository::update(pool, &item).await?;
        
        Ok(item)
    }
    
    pub async fn delete_item(pool: &SqlitePool, user_id: &str, id: &str) -> Result<(), AppError> {
        ClipboardRepository::delete(pool, id, user_id).await
    }
    
    pub async fn search_items(
        pool: &SqlitePool, 
        user_id: &str, 
        query: &str, 
        limit: i64, 
        offset: i64
    ) -> Result<Vec<ClipboardItem>, AppError> {
        ClipboardRepository::search(pool, user_id, query, limit, offset).await
    }
    
    // 解密剪贴板项目
    pub async fn decrypt_item(
        pool: &SqlitePool, 
        user_id: &str, 
        item: &ClipboardItem
    ) -> Result<String, AppError> {
        if !item.encrypted {
            return Ok(item.content.clone());
        }
        
        // 获取用户的加密密钥
        let encryption_key = EncryptionRepository::find_by_user_id(pool, user_id).await?
            .ok_or_else(|| AppError::NotFound("加密密钥不存在".to_string()))?;
        
        // 解码base64
        let combined = base64::decode(&item.content)
            .map_err(|e| AppError::CryptoError(e.to_string()))?;
        
        if combined.len() < 12 {
            return Err(AppError::InvalidData("无效的加密数据".to_string()));
        }
        
        // 分离nonce和加密数据
        let nonce = &combined[0..12];
        let encrypted_data = &combined[12..];
        
        let mut nonce_array = [0u8; 12];
        nonce_array.copy_from_slice(nonce);
        
        // 解密数据
        let decrypted = crypto::decrypt_data(
            encrypted_data,
            &encryption_key.key_data,
            &nonce_array
        ).map_err(|e| AppError::CryptoError(e))?;
        
        Ok(decrypted)
    }
}
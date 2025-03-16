#[cfg(test)]
mod clipboard_tests {
    use crate::clipboard_dao;
    use crate::ClipboardItem;
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
    use std::time::{SystemTime, UNIX_EPOCH};

    // 辅助函数：获取测试数据库连接
    async fn get_test_db() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect(":memory:")
            .await
            .expect("Failed to connect to in-memory SQLite database")
    }

    // 辅助函数：初始化测试数据库
    async fn init_test_db(pool: &SqlitePool) {
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS clipboard_items (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                title TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                is_pinned INTEGER DEFAULT 0
            )
            ",
        )
        .execute(pool)
        .await
        .expect("Failed to create clipboard_items table");

        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS sync_status (
                item_id TEXT PRIMARY KEY,
                is_synced INTEGER DEFAULT 0,
                last_sync_attempt INTEGER,
                FOREIGN KEY (item_id) REFERENCES clipboard_items(id) ON DELETE CASCADE
            )
            ",
        )
        .execute(pool)
        .await
        .expect("Failed to create sync_status table");
    }

    // 测试添加剪贴板项目
    #[tokio::test]
    async fn test_add_item() {
        let pool = get_test_db().await;
        init_test_db(&pool).await;

        let content = "Test content";
        let title = Some("Test title".to_string());

        let result = clipboard_dao::add_item(&pool, content.to_string(), title.clone()).await;
        assert!(result.is_ok(), "添加剪贴板项目失败");

        let item = result.unwrap();
        assert_eq!(item.content, content);
        assert_eq!(item.title, title);
        assert!(!item.is_pinned);
    }

    // 测试获取剪贴板项目
    #[tokio::test]
    async fn test_get_item() {
        let pool = get_test_db().await;
        init_test_db(&pool).await;

        let content = "Test content";
        let title = Some("Test title".to_string());

        let added_item = clipboard_dao::add_item(&pool, content.to_string(), title.clone())
            .await
            .expect("添加剪贴板项目失败");

        let result = clipboard_dao::get_item(&pool, &added_item.id).await;
        assert!(result.is_ok(), "获取剪贴板项目失败");

        let item = result.unwrap();
        assert_eq!(item.id, added_item.id);
        assert_eq!(item.content, content);
        assert_eq!(item.title, title);
    }

    // 测试更新剪贴板项目
    #[tokio::test]
    async fn test_update_item() {
        let pool = get_test_db().await;
        init_test_db(&pool).await;

        let content = "Test content";
        let title = Some("Test title".to_string());

        let added_item = clipboard_dao::add_item(&pool, content.to_string(), title.clone())
            .await
            .expect("添加剪贴板项目失败");

        let new_content = "Updated content";
        let new_title = Some("Updated title".to_string());

        let mut updated_item = added_item.clone();
        updated_item.content = new_content.to_string();
        updated_item.title = new_title.clone();

        let result = clipboard_dao::update_item(&pool, &updated_item).await;
        assert!(result.is_ok(), "更新剪贴板项目失败");

        let item = clipboard_dao::get_item(&pool, &added_item.id)
            .await
            .expect("获取剪贴板项目失败");

        assert_eq!(item.content, new_content);
        assert_eq!(item.title, new_title);
    }

    // 测试删除剪贴板项目
    #[tokio::test]
    async fn test_delete_item() {
        let pool = get_test_db().await;
        init_test_db(&pool).await;

        let content = "Test content";
        let title = Some("Test title".to_string());

        let added_item = clipboard_dao::add_item(&pool, content.to_string(), title.clone())
            .await
            .expect("添加剪贴板项目失败");

        let result = clipboard_dao::delete_item(&pool, &added_item.id).await;
        assert!(result.is_ok(), "删除剪贴板项目失败");

        let result = clipboard_dao::get_item(&pool, &added_item.id).await;
        assert!(result.is_err(), "剪贴板项目应该已被删除");
    }
}

#[cfg(test)]
mod security_tests {
    use crate::security;

    // 测试密码哈希和验证
    #[test]
    fn test_password_hash_verify() {
        let password = "StrongPassword123!";
        
        let hash_result = security::hash_password(password);
        assert!(hash_result.is_ok(), "密码哈希失败");
        
        let hash = hash_result.unwrap();
        let verify_result = security::verify_password(&hash, password);
        
        assert!(verify_result.is_ok(), "密码验证失败");
        assert!(verify_result.unwrap(), "密码应该验证通过");
        
        // 测试错误密码
        let wrong_password = "WrongPassword123!";
        let verify_wrong_result = security::verify_password(&hash, wrong_password);
        
        assert!(verify_wrong_result.is_ok(), "密码验证失败");
        assert!(!verify_wrong_result.unwrap(), "错误密码不应该验证通过");
    }
    
    // 测试数据加密和解密
    #[test]
    fn test_encrypt_decrypt() {
        let data = "Sensitive data that needs encryption";
        
        // 生成加密密钥和nonce
        let key = security::generate_encryption_key();
        let nonce = security::generate_nonce();
        
        // 加密数据
        let encrypted_result = security::encrypt_data(data, &key, &nonce);
        assert!(encrypted_result.is_ok(), "数据加密失败");
        
        let encrypted_data = encrypted_result.unwrap();
        
        // 解密数据
        let decrypted_result = security::decrypt_data(&encrypted_data, &key, &nonce);
        assert!(decrypted_result.is_ok(), "数据解密失败");
        
        let decrypted_data = decrypted_result.unwrap();
        assert_eq!(decrypted_data, data, "解密后的数据应该与原始数据相同");
        
        // 使用错误的密钥尝试解密
        let wrong_key = security::generate_encryption_key();
        let decrypt_wrong_key_result = security::decrypt_data(&encrypted_data, &wrong_key, &nonce);
        assert!(decrypt_wrong_key_result.is_err(), "使用错误密钥不应该成功解密");
    }
}
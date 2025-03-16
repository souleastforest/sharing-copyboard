#[cfg(test)]
mod api_tests {
    use crate::{clipboard_dao, security, account};
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
    use std::sync::Arc;
    
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
        // 创建剪贴板表
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
        
        // 创建同步状态表
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
        
        // 创建用户表
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            ",
        )
        .execute(pool)
        .await
        .expect("Failed to create users table");
        
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
            ",
        )
        .execute(pool)
        .await
        .expect("Failed to create sessions table");
    }
    
    // 测试剪贴板API
    #[tokio::test]
    async fn test_clipboard_api() {
        let pool = get_test_db().await;
        init_test_db(&pool).await;
        
        // 测试添加剪贴板项目
        let content = "API Test Content";
        let title = Some("API Test Title".to_string());
        
        let item = clipboard_dao::add_item(&pool, content.to_string(), title.clone())
            .await
            .expect("添加剪贴板项目失败");
        
        // 测试获取所有剪贴板项目
        let items = clipboard_dao::get_all_items(&pool)
            .await
            .expect("获取所有剪贴板项目失败");
        
        assert!(!items.is_empty(), "剪贴板项目列表不应为空");
        assert_eq!(items.len(), 1, "应该只有一个剪贴板项目");
        assert_eq!(items[0].id, item.id, "剪贴板项目ID应该匹配");
        
        // 测试搜索剪贴板项目
        let search_results = clipboard_dao::search_items(&pool, "Test")
            .await
            .expect("搜索剪贴板项目失败");
        
        assert!(!search_results.is_empty(), "搜索结果不应为空");
        assert_eq!(search_results[0].id, item.id, "搜索结果应该包含添加的项目");
    }
    
    // 测试账户API
    #[tokio::test]
    async fn test_account_api() {
        let pool = get_test_db().await;
        init_test_db(&pool).await;
        
        // 测试用户注册
        let email = "test@example.com";
        let password = "StrongPassword123!";
        
        // 生成验证码
        let verification_code = account::generate_verification_code(&pool, email)
            .await
            .expect("生成验证码失败");
        
        // 注册用户
        let register_request = account::RegisterRequest {
            email: email.to_string(),
            password: password.to_string(),
            verification_code,
        };
        
        let user = account::register_user(&pool, &register_request)
            .await
            .expect("用户注册失败");
        
        assert_eq!(user.email, email, "用户邮箱应该匹配");
        
        // 测试用户登录
        let login_request = account::LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
            remember_me: false,
        };
        
        let session = account::login_user(&pool, &login_request, "test_device")
            .await
            .expect("用户登录失败");
        
        assert_eq!(session.user_id, user.id, "会话用户ID应该匹配");
        assert_eq!(session.device_id, "test_device", "会话设备ID应该匹配");
    }
    
    // 测试安全API
    #[tokio::test]
    async fn test_security_api() {
        // 测试数据加密和解密
        let data = "API Security Test Data";
        
        // 生成加密密钥和nonce
        let key = security::generate_encryption_key();
        let nonce = security::generate_nonce();
        
        // 加密数据
        let encrypted_data = security::encrypt_data(data, &key, &nonce)
            .expect("数据加密失败");
        
        // 解密数据
        let decrypted_data = security::decrypt_data(&encrypted_data, &key, &nonce)
            .expect("数据解密失败");
        
        assert_eq!(decrypted_data, data, "解密后的数据应该与原始数据相同");
    }
}
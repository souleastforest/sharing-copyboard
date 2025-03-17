use crate::entity::user::User;
use crate::error::AppError;
use sqlx::SqlitePool;

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>, AppError> {
        // 使用 query_as_unchecked! 宏来避免类型检查问题
        // 或者确保查询结果中的字段与 User 结构体匹配
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, created_at, updated_at 
                FROM users 
                WHERE email = ?
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, username, created_at, updated_at
               FROM users 
               WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(user)
    }

    pub async fn save(pool: &SqlitePool, user: &User, password_hash: &str) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO users (id, email, username, password_hash, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.username)
        .bind(password_hash)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // 其他数据库操作方法...
}

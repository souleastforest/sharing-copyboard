use crate::entity::clipboard_item::ClipboardItem;
use crate::error::AppError;
use sqlx::SqlitePool;

pub struct ClipboardRepository;

impl ClipboardRepository {
    pub async fn save(pool: &SqlitePool, item: &ClipboardItem) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO clipboard_items (id, user_id, content, content_type, encrypted, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&item.id)
        .bind(&item.user_id)
        .bind(&item.content)
        .bind(&item.content_type)
        .bind(item.encrypted as i32)
        .bind(item.created_at)
        .bind(item.updated_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn update(pool: &SqlitePool, item: &ClipboardItem) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE clipboard_items SET
             content = ?,
             content_type = ?,
             encrypted = ?,
             updated_at = ?
             WHERE id = ? AND user_id = ?",
        )
        .bind(&item.content)
        .bind(&item.content_type)
        .bind(item.encrypted as i32)
        .bind(item.updated_at)
        .bind(&item.id)
        .bind(&item.user_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: &str, user_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM clipboard_items WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn find_by_id(
        pool: &SqlitePool,
        id: &str,
        user_id: &str,
    ) -> Result<Option<ClipboardItem>, AppError> {
        let item = sqlx::query_as::<_, ClipboardItem>(
            "SELECT id, user_id, content, content_type, encrypted as \"encrypted: bool\", created_at, updated_at
             FROM clipboard_items WHERE id = ? AND user_id = ?"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(item)
    }

    pub async fn find_all_by_user_id(
        pool: &SqlitePool,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ClipboardItem>, AppError> {
        let items = sqlx::query_as::<_, ClipboardItem>(
            "SELECT id, user_id, content, content_type, encrypted as \"encrypted: bool\", created_at, updated_at
             FROM clipboard_items WHERE user_id = ? ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        // user_id, limit, offset
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(items)
    }

    pub async fn search(
        pool: &SqlitePool,
        user_id: &str,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ClipboardItem>, AppError> {
        let search_query = format!("%{}%", query);

        let items = sqlx::query_as::<_, ClipboardItem>(
            "SELECT id, user_id, content, content_type, encrypted as \"encrypted: bool\", created_at, updated_at
             FROM clipboard_items 
             WHERE user_id = ? AND content LIKE ? 
             ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        //     user_id, search_query, limit, offset
        .bind(user_id)
        .bind(search_query)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(items)
    }
}

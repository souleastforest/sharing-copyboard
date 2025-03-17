use crate::entity::session::Session;
use crate::error::AppError;
use sqlx::SqlitePool;

pub struct SessionRepository;

impl SessionRepository {
    pub async fn save(pool: &SqlitePool, session: &Session) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO sessions (token, user_id, device_id, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&session.token)
        .bind(&session.user_id)
        .bind(&session.device_id)
        .bind(session.created_at)
        .bind(session.expires_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn find_by_token(
        pool: &SqlitePool,
        token: &str,
    ) -> Result<Option<Session>, AppError> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT token, user_id, device_id, created_at, expires_at 
             FROM sessions WHERE token = ?",
        )
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(session)
    }

    pub async fn delete_by_token(pool: &SqlitePool, token: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn count_by_user_id(pool: &SqlitePool, user_id: &str) -> Result<i64, AppError> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM sessions WHERE user_id = ?",
            user_id
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.count)
    }
}

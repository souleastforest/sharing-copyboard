use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ClipboardItem {
    pub id: String,
    pub user_id: String,
    pub content: String,
    pub content_type: String,
    pub encrypted: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardItemRequest {
    pub content: String,
    pub content_type: String,
    pub encrypt: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardItemUpdateRequest {
    pub id: String,
    pub content: String,
    pub content_type: String,
    pub encrypt: bool,
}

impl ClipboardItem {
    pub fn new(user_id: &str, content: &str, content_type: &str, encrypted: bool) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            content: content.to_string(),
            content_type: content_type.to_string(),
            encrypted,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self.created_at)
            .unwrap_or_else(|| Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap())
    }

    pub fn updated_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self.updated_at)
            .unwrap_or_else(|| Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap())
    }

    pub fn set_created_at_from_datetime(&mut self, dt: DateTime<Utc>) {
        self.created_at = dt.timestamp_millis();
    }

    pub fn set_updated_at_from_datetime(&mut self, dt: DateTime<Utc>) {
        self.updated_at = dt.timestamp_millis();
    }
}


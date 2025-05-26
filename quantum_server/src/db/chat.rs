use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, FromRow)]
pub struct Chat {
    pub id: Uuid,
    pub name: Option<String>,
    pub chat_type: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct NewChat {
    pub name: Option<String>,
    pub chat_type: String, // "private" или "group"
}
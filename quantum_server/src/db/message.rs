use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use sqlx::PgPool;

#[derive(Debug, Serialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub is_edited: bool,
    pub is_deleted: bool,
}

#[derive(Debug, Deserialize)]
pub struct NewMessage {
    pub chat_id: Uuid,
    pub body: String,
}

pub async fn send_message(
    msg: NewMessage,
    sender_id: Uuid,
    pool: &PgPool
) -> Result<Message, sqlx::Error> {
    let result = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (chat_id, sender_id, body)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(msg.chat_id)
    .bind(sender_id)
    .bind(&msg.body)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn get_messages_for_chat(
    chat_id: Uuid,
    limit: i64,
    pool: &PgPool,
) -> Result<Vec<Message>, sqlx::Error> {
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT * FROM messages
        WHERE chat_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(chat_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}
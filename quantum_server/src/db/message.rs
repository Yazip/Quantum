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
    pub reply_to_id: Option<Uuid>,
    pub forwarded_from: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct NewMessage {
    pub chat_id: Uuid,
    pub body: String,
    pub reply_to_id: Option<Uuid>,
    pub forwarded_from: Option<Uuid>,
}

pub async fn send_message(
    msg: NewMessage,
    sender_id: Uuid,
    pool: &PgPool
) -> Result<Message, sqlx::Error> {
    let result = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (chat_id, sender_id, body, reply_to_id, forwarded_from)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(msg.chat_id)
    .bind(sender_id)
    .bind(&msg.body)
    .bind(msg.reply_to_id)
    .bind(msg.forwarded_from)
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

pub async fn edit_message(
    message_id: Uuid,
    user_id: Uuid,
    new_body: String,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE messages
        SET body = $1, is_edited = true
        WHERE id = $2 AND sender_id = $3
        "#,
        new_body,
        message_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_message(
    message_id: Uuid,
    user_id: Uuid,
    for_all: bool,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let query = if for_all {
        r#"
        UPDATE messages
        SET is_deleted = true, body = '[deleted]'
        WHERE id = $1 AND sender_id = $2
        "#
    } else {
        r#"
        UPDATE messages
        SET is_deleted = true
        WHERE id = $1 AND sender_id = $2
        "#
    };

    sqlx::query(query)
        .bind(message_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn forward_message(
    chat_id: Uuid,
    original_message_id: Uuid,
    sender_id: Uuid,
    pool: &PgPool,
) -> Result<Message, sqlx::Error> {
    let original = sqlx::query!(
        r#"
        SELECT body, sender_id
        FROM messages
        WHERE id = $1
        "#,
        original_message_id
    )
    .fetch_one(pool)
    .await?;

    let forwarded = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (chat_id, sender_id, body, forwarded_from)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(chat_id)
    .bind(sender_id)
    .bind(original.body)
    .bind(original.sender_id)
    .fetch_one(pool)
    .await?;

    Ok(forwarded)
}
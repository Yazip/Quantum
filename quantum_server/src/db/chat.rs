use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use sqlx::PgPool;

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

pub async fn create_group_chat(
    name: String,
    creator_id: Uuid,
    member_ids: Vec<Uuid>,
    pool: &PgPool,
) -> Result<Uuid, sqlx::Error> {
    let chat_id = sqlx::query_scalar!(
        r#"
        INSERT INTO chats (name, chat_type)
        VALUES ($1, 'group')
        RETURNING id
        "#,
        name
    )
    .fetch_one(pool)
    .await?;

    let mut members = member_ids;
    members.push(creator_id); // гарантируем, что создатель в чате

    for user_id in members {
        sqlx::query!(
            r#"
            INSERT INTO chat_members (chat_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            chat_id,
            user_id
        )
        .execute(pool)
        .await?;
    }

    Ok(chat_id)
}

pub async fn add_user_to_chat(
    chat_id: Uuid,
    user_id: Uuid,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO chat_members (chat_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
        chat_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_user_from_chat(
    chat_id: Uuid,
    user_id: Uuid,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM chat_members
        WHERE chat_id = $1 AND user_id = $2
        "#,
        chat_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_chat_members(
    chat_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let records = sqlx::query!(
        r#"
        SELECT user_id FROM chat_members
        WHERE chat_id = $1
        "#,
        chat_id
    )
    .fetch_all(pool)
    .await?;

    Ok(records.into_iter().map(|r| r.user_id).collect())
}

pub async fn is_user_in_chat(
    chat_id: Uuid,
    user_id: Uuid,
    pool: &PgPool,
) -> Result<bool, sqlx::Error> {
    let record = sqlx::query_scalar!(
        r#"
        SELECT 1 FROM chat_members
        WHERE chat_id = $1 AND user_id = $2
        "#,
        chat_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.is_some())
}
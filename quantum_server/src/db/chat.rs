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

#[derive(Debug, Serialize)]
pub struct UserChat {
    pub id: Uuid,
    pub name: Option<String>,
    pub chat_type: String,
}

pub async fn create_group_chat(
    name: String,
    chat_type: &str,
    creator_id: Uuid,
    member_ids: Vec<Uuid>,
    pool: &PgPool,
) -> Result<Uuid, sqlx::Error> {
    let chat_id = sqlx::query_scalar!(
        r#"
        INSERT INTO chats (name, chat_type)
        VALUES ($1, $2)
        RETURNING id
        "#,
        name,
        chat_type
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

pub async fn get_user_chats(user_id: Uuid, pool: &PgPool) -> Result<Vec<UserChat>, sqlx::Error> {
    let rows = sqlx::query_as!(
        UserChat,
        r#"
        SELECT c.id, c.name, c.chat_type
        FROM chats c
        JOIN chat_members m ON c.id = m.chat_id
        WHERE m.user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn find_private_chat_between(user1: Uuid, user2: Uuid, pool: &PgPool) -> Result<Option<Uuid>, sqlx::Error> {
    let record = sqlx::query!(
        r#"
        SELECT c.id
        FROM chats c
        JOIN chat_members cm1 ON cm1.chat_id = c.id AND cm1.user_id = $1
        JOIN chat_members cm2 ON cm2.chat_id = c.id AND cm2.user_id = $2
        WHERE c.chat_type = 'private'
        "#,
        user1,
        user2
    )
    .fetch_optional(pool)
    .await?;

    Ok(record.map(|r| r.id))
}

pub async fn get_chat_name_by_id(chat_id: Uuid, pool: &PgPool) -> Result<String, sqlx::Error> {
    let name_opt = sqlx::query_scalar!(
        r#"SELECT name FROM chats WHERE id = $1"#,
        chat_id
    )
    .fetch_one(pool)
    .await?;

    if let Some(name) = name_opt {
        Ok(name)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}
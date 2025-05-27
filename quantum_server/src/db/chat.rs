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
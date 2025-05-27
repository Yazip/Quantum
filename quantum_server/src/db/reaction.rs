use serde::Deserialize;
use uuid::Uuid;
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct Reaction {
    pub message_id: Uuid,
    pub emoji: String,
}

pub async fn set_reaction(
    message_id: Uuid,
    user_id: Uuid,
    emoji: String,
    pool: &PgPool
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO message_reactions (message_id, user_id, emoji)
        VALUES ($1, $2, $3)
        ON CONFLICT (message_id, user_id)
        DO UPDATE SET emoji = $3
        "#,
        message_id,
        user_id,
        emoji
    )
    .execute(pool)
    .await?;

    Ok(())
}
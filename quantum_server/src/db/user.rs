use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use sqlx::PgPool;
use bcrypt::{hash, DEFAULT_COST};

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub public_key: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub public_key: String,
}

pub async fn create_user(new_user: NewUser, pool: &PgPool) -> Result<User, sqlx::Error> {
    let hashed = hash(&new_user.password, DEFAULT_COST).expect("Failed to hash password");

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, password_hash, public_key)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&new_user.username)
    .bind(&hashed)
    .bind(&new_user.public_key)
    .fetch_one(pool)
    .await?;

    Ok(user)
}
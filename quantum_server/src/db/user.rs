use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;
use sqlx::PgPool;
use bcrypt::{hash, DEFAULT_COST};
use bcrypt::verify;
use super::*;
use dotenvy::dotenv;
use crate::db::init_pool;

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

pub async fn find_user_by_username(username: &str, pool: &PgPool) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn authenticate_user(
    username: &str,
    password: &str,
    pool: &PgPool,
) -> Result<User, String> {
    let user = find_user_by_username(username, pool)
        .await
        .map_err(|_| "Пользователь не найден".to_string())?;

    let valid = verify(password, &user.password_hash)
        .map_err(|_| "Ошибка при проверке пароля".to_string())?;

    if valid {
        Ok(user)
    } else {
        Err("Неверный пароль".to_string())
    }
}

pub async fn user_exists(user_id: Uuid, pool: &PgPool) -> Result<bool, sqlx::Error> {
    let record = sqlx::query_scalar!(
        "SELECT 1 FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(record.is_some())
}

pub async fn get_username_by_id(user_id: &Uuid, pool: &PgPool) -> Result<String, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT username FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(row.username)
}
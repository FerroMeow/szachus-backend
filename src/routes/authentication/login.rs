use anyhow::anyhow;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Postgres};

use crate::{error::AppError, ServerState};

use super::jwt::create_token;

#[derive(Deserialize)]
pub struct UserCredentials {
    username: String,
    password: String,
}

struct PlayerEntity {
    id: i32,
    password_hash: String,
}

pub async fn on_post(
    State(server_state): State<ServerState>,
    Json(credentials): Json<UserCredentials>,
) -> Result<Json<serde_json::Value>, AppError> {
    let jwt = authenticate_user(&server_state.db_pool, &credentials).await?;
    Ok(Json(json!({
        "jwt": jwt,
    })))
}

async fn authenticate_user(
    db_pool: &Pool<Postgres>,
    credentials: &UserCredentials,
) -> anyhow::Result<String> {
    let player = sqlx::query_as!(
        PlayerEntity,
        "SELECT id, password_hash FROM player WHERE username = $1",
        credentials.username
    )
    .fetch_optional(db_pool)
    .await?
    .ok_or(anyhow!("No player found"))?;

    let parsed_hash = PasswordHash::new(&player.password_hash)?;
    Argon2::default().verify_password(credentials.password.as_bytes(), &parsed_hash)?;
    create_token(player.id)
}

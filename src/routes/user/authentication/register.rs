use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use axum::{extract::State, Json};
use rand::rngs::OsRng;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Postgres};

use crate::{error, routes::user::authentication::jwt::create_token, ServerState};

#[derive(Deserialize)]
pub struct UserData {
    username: String,
    password: String,
}

pub async fn on_register(
    State(server_state): State<ServerState>,
    Json(user_data): Json<UserData>,
) -> error::Result<Json<serde_json::Value>> {
    let jwt = create_user(&server_state.db_pool, &user_data).await?;
    Ok(Json(json!({
        "jwt": jwt
    })))
}

async fn create_user(db_pool: &Pool<Postgres>, user_data: &UserData) -> anyhow::Result<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(OsRng);
    let password_hash = argon2.hash_password(user_data.password.as_bytes(), &salt)?;
    sqlx::query!(
        "INSERT INTO player (username, password_hash, salt) VALUES ($1, $2, $3)",
        user_data.username,
        password_hash.to_string(),
        salt.to_string()
    )
    .execute(db_pool)
    .await?;
    create_token(&user_data.username)
}

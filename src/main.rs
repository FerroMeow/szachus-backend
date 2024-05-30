use axum::{routing::get, Router};
use dotenv::dotenv;
use game::ws_handler;
use sqlx::{Pool, Postgres};
use std::{env, fs};
use user::user_routes;

mod error;
mod game;
mod user;

#[derive(Clone)]
struct ServerState {
    db_pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let password_file = env::var("PASSWORD_FILE").unwrap_or("db/dev.password.txt".to_owned());
    let password = String::from_utf8(fs::read(password_file)?)?;
    let connection_string = format!("postgres://postgres:{}@localhost/szachus", &password);
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .unwrap();
    let app_routes = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws/gameplay", get(ws_handler))
        .nest("/user", user_routes())
        .with_state(ServerState { db_pool });
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        app_routes,
    )
    .await
    .unwrap();
    Ok(())
}

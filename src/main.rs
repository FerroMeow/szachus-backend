use axum::{routing::get, Router};
use game::ws_handler;
use std::{env, fs};

mod game;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let password_file = env::var("PASSWORD_FILE").unwrap_or("db/dev.password.txt".to_owned());
    let password = String::from_utf8(fs::read(password_file)?)?;
    let connection_string = format!("postgres://postgres:{}@localhost/szachus", &password);
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string);
    let app_routes = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws/gameplay", get(ws_handler));
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        app_routes,
    )
    .await
    .unwrap();
    Ok(())
}

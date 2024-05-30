use dotenv::dotenv;
use sqlx::{Pool, Postgres};
use std::{env, fs};

mod error;
mod routes;

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
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(),
        routes::app_routes().with_state(ServerState { db_pool }),
    )
    .await
    .unwrap();
    Ok(())
}

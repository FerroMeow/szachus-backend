use axum::extract::FromRef;
use dotenv::dotenv;
use handlebars::Handlebars;
use routes::game::matchmaking::matchmaking_state::UserQueue;
use sqlx::{Pool, Postgres};
use std::{env, fs};

mod error;
mod routes;
mod templates;

#[derive(Clone)]
struct GlobalState {
    pub db_pool: Pool<Postgres>,
}

impl FromRef<ServerState> for GlobalState {
    fn from_ref(input: &ServerState) -> Self {
        input.global.clone()
    }
}

#[derive(Clone)]
struct ServerState {
    global: GlobalState,
    user_queue: UserQueue,
    handlebars: Handlebars<'static>,
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
        routes::app_routes().with_state(ServerState {
            global: GlobalState { db_pool },
            user_queue: UserQueue::default(),
            handlebars: Handlebars::new(),
        }),
    )
    .await
    .unwrap();
    Ok(())
}

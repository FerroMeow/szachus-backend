use axum::{routing::get, Router};

use crate::ServerState;

mod game;
mod user;

pub fn app_routes() -> Router<ServerState> {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws/gameplay", get(game::ws_handler))
        .nest("/user", user::user_routes())
}

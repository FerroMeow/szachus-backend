use axum::{middleware, routing::get, Router};

use crate::ServerState;

pub mod game;
mod user;

pub fn app_routes() -> Router<ServerState> {
    Router::new()
        .route("/", get(|| async { "Hello, World 2!" }))
        .nest("/game", game::routes())
        .layer(middleware::from_extractor::<user::jwt::Claims>())
        .nest("/user", user::authentication_routes())
}

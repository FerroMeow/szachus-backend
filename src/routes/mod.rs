use axum::{middleware, routing::get, Router};

use crate::ServerState;

mod authentication;
mod game;
mod user;

pub fn app_routes() -> Router<ServerState> {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws/gameplay", get(game::ws_handler))
        .nest("/user", user::user_routes())
        .layer(middleware::from_extractor::<authentication::jwt::Claims>())
        .nest(
            "/user/authentication",
            authentication::authentication_routes(),
        )
}

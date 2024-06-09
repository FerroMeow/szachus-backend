use axum::{middleware, routing::get, Router};

use crate::ServerState;

mod authentication;
mod game;

pub fn app_routes() -> Router<ServerState> {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/game", game::routes())
        .layer(middleware::from_extractor::<authentication::jwt::Claims>())
        .nest(
            "/user/authentication",
            authentication::authentication_routes(),
        )
}

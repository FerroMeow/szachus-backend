use axum::{routing::post, Router};

use crate::ServerState;

mod authentication;
mod jwt;

pub fn user_routes() -> Router<ServerState> {
    Router::new().route("/authentication", post(authentication::on_post))
}

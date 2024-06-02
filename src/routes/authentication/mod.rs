use axum::{routing::post, Router};

use crate::ServerState;

pub mod jwt;
mod login;
mod register;

pub fn authentication_routes() -> Router<ServerState> {
    Router::new()
        .route("/login", post(login::on_post))
        .route("/register", post(register::on_register))
}

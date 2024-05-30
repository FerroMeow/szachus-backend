use authentication::authentication_routes;
use axum::Router;

use crate::ServerState;

mod authentication;

pub fn user_routes() -> Router<ServerState> {
    Router::new().nest("/authentication", authentication_routes())
}

use axum::Router;

use crate::ServerState;

pub fn user_routes() -> Router<ServerState> {
    Router::new()
}

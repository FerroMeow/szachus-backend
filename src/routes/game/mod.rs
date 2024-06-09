use axum::{
    routing::{get, post},
    Router,
};

use crate::ServerState;

mod matchmaking;
mod turn;

pub fn routes() -> Router<ServerState> {
    Router::new()
        // Matchmaking WebSocket, dropped when match found
        .route("/", post(matchmaking::route_handler))
        // Game lifecycle websocket
        .route("/:id/turn/", get(turn::route_handler))
}

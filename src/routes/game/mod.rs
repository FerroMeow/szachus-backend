use axum::{routing::get, Router};

use crate::ServerState;

pub mod gameplay;
pub mod matchmaking;
pub mod opponent_pair;
pub mod piece_color;
pub mod ws;
pub mod ws_messages;

pub fn routes() -> Router<ServerState> {
    Router::new()
        // Matchmaking WebSocket, dropped when match found
        .route("/", get(matchmaking::route_handler))
}

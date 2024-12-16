use axum::{routing::get, Router};

use crate::ServerState;

pub mod chessboard;
pub mod gameplay;
pub mod matchmaking;
pub mod opponent_pair;
pub mod piece;
pub mod position;
pub mod ws;
pub mod ws_messages;

pub fn routes() -> Router<ServerState> {
    Router::new()
        // Matchmaking WebSocket, dropped when match found
        .route("/", get(matchmaking::ws_handler::route_handler))
}

use std::collections::VecDeque;

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};

use crate::ServerState;

mod matchmaking;
mod turn;

#[derive(Default, Clone)]
pub struct MatchmakingState(pub VecDeque<i32>);

impl FromRef<ServerState> for MatchmakingState {
    fn from_ref(input: &ServerState) -> Self {
        input.user_queue.clone()
    }
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        // Matchmaking WebSocket, dropped when match found
        .route("/", post(matchmaking::route_handler))
        // Game lifecycle websocket
        .route("/:id/turn/", get(turn::route_handler))
}

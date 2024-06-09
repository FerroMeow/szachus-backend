use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        FromRef,
    },
    routing::get,
    Router,
};
use futures::{lock::Mutex, stream::SplitSink};

use crate::ServerState;

pub mod matchmaking;
mod turn;

type MatchmakingInnerState = SplitSink<WebSocket, Message>;

#[derive(Default, Clone)]
pub struct MatchmakingState(pub Arc<Mutex<HashMap<i32, Arc<Mutex<MatchmakingInnerState>>>>>);

impl FromRef<ServerState> for MatchmakingState {
    fn from_ref(input: &ServerState) -> Self {
        input.user_queue.clone()
    }
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        // Matchmaking WebSocket, dropped when match found
        .route("/", get(matchmaking::route_handler))
        // Game lifecycle websocket
        .route("/:id/turn/", get(turn::route_handler))
}

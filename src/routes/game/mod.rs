use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        FromRef,
    },
    routing::get,
    Router,
};
use futures::{lock::Mutex, stream};
use matchmaking::Game;
use tokio::task::JoinHandle;

use crate::ServerState;

pub mod gameplay;
pub mod matchmaking;
pub mod rules;

pub type SplitSink = stream::SplitSink<WebSocket, Message>;
pub type SplitStream = stream::SplitStream<WebSocket>;

#[derive(Default, Clone)]
pub struct MatchmakingState(
    pub  Arc<
        Mutex<
            HashMap<
                i32,
                (
                    Arc<Mutex<SplitSink>>,
                    Arc<Mutex<SplitStream>>,
                    Arc<JoinHandle<()>>,
                ),
            >,
        >,
    >,
);

impl FromRef<ServerState> for MatchmakingState {
    fn from_ref(input: &ServerState) -> Self {
        input.user_queue.clone()
    }
}

pub struct OpenGame {
    pub game_data: Game,
    pub user_stream: (
        (Arc<Mutex<SplitSink>>, Arc<Mutex<SplitStream>>),
        (Arc<Mutex<SplitSink>>, Arc<Mutex<SplitStream>>),
    ),
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        // Matchmaking WebSocket, dropped when match found
        .route("/", get(matchmaking::route_handler))
}

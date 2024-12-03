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
use gameplay::GameMessage;
use matchmaking::{Game, MatchmakingResponse};
use rules::ChessBoard;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::ServerState;

pub mod gameplay;
pub mod matchmaking;
pub mod rules;

pub type SplitSink = stream::SplitSink<WebSocket, Message>;
pub type SplitStream = stream::SplitStream<WebSocket>;

type PlayerState = (ArcMut<SplitSink>, ArcMut<SplitStream>, Arc<JoinHandle<()>>);

#[derive(Default, Clone)]
pub struct MatchmakingState(pub ArcMut<HashMap<i32, PlayerState>>);

impl FromRef<ServerState> for MatchmakingState {
    fn from_ref(input: &ServerState) -> Self {
        input.user_queue.clone()
    }
}

type ArcMut<T> = Arc<Mutex<T>>;

// txc, rx
type SinkStream = (ArcMut<SplitSink>, ArcMut<SplitStream>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum WsMsg {
    Matchmaking(MatchmakingResponse),
    Game(GameMessage),
}

// player tx, rx
pub struct PlayerStreams {
    pub white_player: SinkStream,
    pub black_player: SinkStream,
}

pub struct OpenGame {
    pub game_data: Game,
    pub chess_board: Arc<Mutex<ChessBoard>>,
    pub user_stream: PlayerStreams,
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        // Matchmaking WebSocket, dropped when match found
        .route("/", get(matchmaking::route_handler))
}

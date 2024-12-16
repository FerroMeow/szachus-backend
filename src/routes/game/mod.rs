use axum::{routing::get, Router};
use chessboard::ChessBoard;
use matchmaking::Game;
use ws::GameWs;

use crate::ServerState;

pub mod chessboard;
pub mod gameplay;
pub mod matchmaking;
pub mod piece;
pub mod position;
pub mod ws;
pub mod ws_messages;

// player tx, rx
pub struct PlayerStreams {
    pub white_player: GameWs,
    pub black_player: GameWs,
}

pub struct OpenGame {
    pub game_data: Game,
    pub chess_board: ChessBoard,
    pub user_stream: PlayerStreams,
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        // Matchmaking WebSocket, dropped when match found
        .route("/", get(matchmaking::ws_handler::route_handler))
}

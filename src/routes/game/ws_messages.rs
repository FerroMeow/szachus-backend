use serde::{Deserialize, Serialize};

use super::{matchmaking::ws_handler::MatchmakingServerMsg, piece::PieceColor, position::Position};

#[derive(Deserialize, Debug, Serialize, Clone, Copy)]
pub struct ChessMove {
    pub position_from: Position,
    pub position_to: Position,
}

#[derive(Deserialize, Debug)]
pub(crate) enum GameClientMsg {
    TurnEnd(ChessMove),
    Ack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum GameServerMsg {
    NewTurn(bool),
    Error(String),
    MovedCorrectly(Option<(PieceColor, Position)>),
    GameEnd(bool),
    PawnMove(ChessMove, Option<(PieceColor, Position)>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ServerMsg {
    Matchmaking(MatchmakingServerMsg),
    Game(GameServerMsg),
}

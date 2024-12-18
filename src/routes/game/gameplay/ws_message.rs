use serde::{Deserialize, Serialize};

use crate::routes::game::{piece_color::PieceColor, ws_messages::ChessMove};

use super::position::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum GameServerMsg {
    NewTurn(bool),
    Error(String),
    GameEnd(bool),
    PawnMove(ChessMove, Option<(PieceColor, Position)>),
}

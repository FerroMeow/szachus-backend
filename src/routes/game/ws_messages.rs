use serde::{Deserialize, Serialize};

use super::{
    gameplay::{position::Position, ws_message::GameServerMsg},
    matchmaking::ws_message::MatchmakingServerMsg,
    piece_color::PieceColor,
};

#[derive(Deserialize, Debug, Serialize, Clone, Copy)]
pub struct ChessMove {
    pub position_from: Position,
    pub position_to: Position,
}

impl ChessMove {
    pub fn invert(&self) -> Self {
        ChessMove {
            position_from: self.position_from.invert(),
            position_to: self.position_to.invert(),
        }
    }
    pub fn maybe_invert(&self, color: PieceColor) -> Self {
        match color {
            PieceColor::Black => self.invert(),
            PieceColor::White => *self,
        }
    }
}

#[derive(Deserialize, Debug)]
pub(crate) enum GameClientMsg {
    TurnEnd(ChessMove),
    Ack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ServerMsg {
    Matchmaking(MatchmakingServerMsg),
    Game(GameServerMsg),
}

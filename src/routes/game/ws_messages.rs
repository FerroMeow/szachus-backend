use serde::{Deserialize, Serialize};

use super::{piece::PieceColor, position::Position, MatchmakingMsgSend};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ChessMove {
    pub position_from: Position,
    pub position_to: Position,
}

#[derive(Deserialize, Debug)]
pub(crate) enum GameMsgRecv {
    TurnEnd(ChessMove),
    Ack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum GameMsgSend {
    NewTurn(bool),
    Error(String),
    MovedCorrectly(Option<(PieceColor, Position)>),
    GameEnd(bool),
    PawnMove(ChessMove, Option<(PieceColor, Position)>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum WsMsgSend {
    Matchmaking(MatchmakingMsgSend),
    Game(GameMsgSend),
}

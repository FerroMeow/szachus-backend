use serde::{Deserialize, Serialize};

use super::{
    gameplay::{position::Position, ws_message::GameServerMsg},
    matchmaking::ws_message::MatchmakingServerMsg,
};

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
pub(crate) enum ServerMsg {
    Matchmaking(MatchmakingServerMsg),
    Game(GameServerMsg),
}

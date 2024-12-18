use serde::{Deserialize, Serialize};

use crate::routes::game::piece_color::PieceColor;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum MatchmakingServerMsg {
    Searching,
    Success { color: PieceColor },
    Error(String),
    GameDropped(String),
}

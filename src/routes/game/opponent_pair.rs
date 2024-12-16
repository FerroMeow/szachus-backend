use std::mem;

use super::{piece::PieceColor, ws::GameWs};

#[derive(Debug)]
pub struct OpponentPair {
    pub white_player: GameWs,
    pub black_player: GameWs,
    pub current_player_color: PieceColor,
}

impl OpponentPair {
    pub fn new(white_player: GameWs, black_player: GameWs) -> Self {
        Self {
            white_player,
            black_player,
            current_player_color: PieceColor::White,
        }
    }

    pub fn get_active(&self) -> &GameWs {
        match self.current_player_color {
            PieceColor::White => &self.white_player,
            PieceColor::Black => &self.black_player,
        }
    }

    pub fn get_passive(&self) -> &GameWs {
        match self.current_player_color {
            PieceColor::White => &self.black_player,
            PieceColor::Black => &self.white_player,
        }
    }

    pub fn switch_active(&mut self) -> &mut Self {
        mem::swap(&mut self.white_player, &mut self.black_player);
        self
    }
}

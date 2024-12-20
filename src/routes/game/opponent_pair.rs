use super::{
    gameplay::player::GamePlayer, matchmaking::matchmaking_state::MatchmakingPlayer,
    piece_color::PieceColor,
};

#[derive(Debug)]
pub struct OpponentPair {
    pub white_player: GamePlayer,
    pub black_player: GamePlayer,
    pub current_player_color: PieceColor,
}

impl OpponentPair {
    pub fn new(white_player: MatchmakingPlayer, black_player: MatchmakingPlayer) -> Self {
        Self {
            white_player: GamePlayer::new(white_player.id, white_player.ws),
            black_player: GamePlayer::new(black_player.id, black_player.ws),
            current_player_color: PieceColor::White,
        }
    }

    pub fn get_active(&self) -> &GamePlayer {
        match self.current_player_color {
            PieceColor::White => &self.white_player,
            PieceColor::Black => &self.black_player,
        }
    }

    pub fn get_passive(&self) -> &GamePlayer {
        match self.current_player_color {
            PieceColor::White => &self.black_player,
            PieceColor::Black => &self.white_player,
        }
    }

    pub fn switch_active(&mut self) -> &mut Self {
        self.current_player_color = self.current_player_color.invert();
        self
    }
}

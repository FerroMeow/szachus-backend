use anyhow::{anyhow, bail};

use crate::routes::game::piece_color::PieceColor;

use super::{
    piece::{Piece, PieceType},
    position::Position,
};

#[derive(Clone, Debug)]
pub struct ChessBoard {
    pub pieces: Vec<Piece>,
}

impl ChessBoard {
    pub fn new() -> Self {
        let mut pieces: Vec<Piece> = Vec::with_capacity(32);
        for color in [PieceColor::White, PieceColor::Black] {
            for column in 0..8 {
                pieces.push(Piece::new(PieceType::Pawn, color, column));
            }
            pieces.push(Piece::new(PieceType::Rook, color, 0));
            pieces.push(Piece::new(PieceType::Rook, color, 7));
            pieces.push(Piece::new(PieceType::Knight, color, 1));
            pieces.push(Piece::new(PieceType::Knight, color, 6));
            pieces.push(Piece::new(PieceType::Bishop, color, 2));
            pieces.push(Piece::new(PieceType::Bishop, color, 5));
            pieces.push(Piece::new(PieceType::Queen, color, 3));
            pieces.push(Piece::new(PieceType::King, color, 4));
        }
        Self { pieces }
    }

    pub fn find_own_piece_at_mut(&mut self, position: Position) -> Option<&mut Piece> {
        self.pieces
            .iter_mut()
            .find(move |piece| piece.position == position)
    }

    pub fn find_king(&self, color: PieceColor) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|piece| piece.color == color && piece.piece_type == PieceType::King)
    }

    pub fn remove_piece(&mut self, position: Position, color: PieceColor) -> Option<Piece> {
        let position = self.pieces.iter().position(move |current_piece| {
            current_piece.position == position && current_piece.color == color
        })?;
        Some(self.pieces.swap_remove(position))
    }

    pub fn is_path_clear(&self, from: Position, to: Position) -> bool {
        // Movement in the same column
        if from.column == to.column {
            for piece in self.pieces.iter() {
                if piece.position.column != from.column {
                    continue;
                }
                let row = piece.position.row;
                if (row > from.row && row < to.row) || (row > to.row && row < from.row) {
                    return false;
                }
            }
        };
        // Movement in the same row
        if from.row == to.row {
            for piece in self.pieces.iter() {
                if piece.position.row != from.row {
                    continue;
                }
                let col = piece.position.column;
                if (col > from.column && col < to.column) || (col > to.column && col < from.column)
                {
                    return false;
                }
            }
        };
        // Diagonal
        let diff_col = (from.column - to.column).abs();
        let diff_row = (from.row - to.row).abs();
        if diff_col == diff_row {
            for i in 1..diff_col {
                let pos = if from.column < to.column && from.row < to.row {
                    (from.column + i, from.row + i)
                } else if from.column < to.column && from.row > to.row {
                    (from.column + i, from.row - i)
                } else if from.column > to.column && from.row < to.row {
                    (from.column - i, from.row + i)
                } else {
                    (from.column - i, from.row - i)
                };
                if self
                    .pieces
                    .iter()
                    .any(|piece| piece.position.column == pos.0 && piece.position.row == pos.1)
                {
                    return false;
                }
            }
        }
        true
    }

    pub async fn move_piece(
        &mut self,
        player_color: PieceColor,
        from: Position,
        to: Position,
    ) -> anyhow::Result<(PieceType, Option<Piece>)> {
        if !self.is_path_clear(from, to) {
            bail!("The path is currently occupied");
        }
        let piece = self.find_own_piece_at_mut(from);
        let piece = piece.ok_or(anyhow!("You don't have a piece at position {from:?}"))?;
        let piece_type = piece.piece_type;
        piece.move_piece_to(to).await?;
        let removed = self.remove_piece(to, player_color.invert());
        Ok((piece_type, removed))
    }
}

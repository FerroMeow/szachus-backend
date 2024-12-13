use anyhow::{anyhow, bail};
use serde::{Deserialize, Serialize};

use super::position::Position;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    pub fn invert(&self) -> Self {
        match *self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PieceColor,
    pub position: Position,
    pub times_moved: u8,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: PieceColor, column: i8) -> anyhow::Result<Piece> {
        let row = match (&piece_type, &color) {
            (PieceType::Pawn, PieceColor::White) => 1,
            (PieceType::Pawn, PieceColor::Black) => 6,
            (_, PieceColor::White) => 0,
            (_, PieceColor::Black) => 7,
        };
        Ok(Piece {
            piece_type,
            color,
            position: Position::new(column, row),
            times_moved: 0,
        })
    }

    pub async fn move_piece_to(&mut self, new_position: &Position) -> anyhow::Result<()> {
        let position_difference = new_position.clone() - self.position.clone();
        match self.piece_type {
            PieceType::Pawn => {
                self.pawn_move(new_position.clone(), position_difference)
                    .await?;
                match self.color {
                    PieceColor::White if new_position.row == 7 => {
                        self.piece_type = PieceType::Queen;
                    }
                    PieceColor::Black if new_position.row == 0 => {
                        self.piece_type = PieceType::Queen;
                    }
                    _ => (),
                };
            }
            PieceType::Knight => {
                if !((position_difference.0.abs() == 1 && position_difference.1.abs() == 2)
                    || (position_difference.0.abs() == 2 && position_difference.1.abs() == 1))
                {
                    bail!("Incorrect knight move!");
                }
            }
            PieceType::King => {
                if !(position_difference.0.abs() <= 1 && position_difference.1.abs() <= 1) {
                    bail!("Incorrect King move!");
                }
            }
            PieceType::Rook => {
                self.rook_move(position_difference).await?;
            }
            PieceType::Bishop => {
                self.bishop_move(position_difference).await?;
            }
            PieceType::Queen => {
                let move_successful = [
                    self.rook_move(position_difference).await,
                    self.bishop_move(position_difference).await,
                ]
                .iter()
                .any(|result| result.is_ok());
                if !move_successful {
                    bail!("Incorrect queen move");
                }
            }
        };
        self.position = new_position.clone();
        self.times_moved += 1;
        Ok(())
    }

    async fn rook_move(&mut self, position_difference: (i8, i8)) -> Result<(), anyhow::Error> {
        if position_difference.0 != 0 && position_difference.1 != 0 {
            bail!("Incorrect rook move");
        }
        Ok(())
    }

    async fn bishop_move(&mut self, position_difference: (i8, i8)) -> Result<(), anyhow::Error> {
        if position_difference.0 == 0
            || position_difference.1 == 0
            || position_difference.0.abs() != position_difference.1.abs()
        {
            bail!("Incorrect bishop move");
        };
        Ok(())
    }

    async fn pawn_move(
        &mut self,
        new_position: Position,
        position_difference: (i8, i8),
    ) -> anyhow::Result<()> {
        if self.position.column == new_position.column {
            if position_difference.0 > 2 {
                bail!("Can't got further than two tiles");
            };
            if position_difference.0 < 1 {
                bail!("Can't got les than one tile");
            };
            if self.times_moved != 0 && position_difference.0 == 2 {
                bail!("Can't move more than one tile after the first move!");
            };
            return Ok(());
        };
        if position_difference.1.abs() != 1 {
            bail!("Movement unreachable by any means");
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ChessBoard {
    pub pieces: Vec<Piece>,
}

impl ChessBoard {
    pub fn new() -> anyhow::Result<Self> {
        let pieces = [PieceColor::White, PieceColor::Black]
            .into_iter()
            .map(|color| {
                (0..8)
                    .map(|column| Piece::new(PieceType::Pawn, color.clone(), column))
                    .chain(
                        [
                            Piece::new(PieceType::Rook, color.clone(), 0),
                            Piece::new(PieceType::Rook, color.clone(), 7),
                            Piece::new(PieceType::Knight, color.clone(), 1),
                            Piece::new(PieceType::Knight, color.clone(), 6),
                            Piece::new(PieceType::Bishop, color.clone(), 2),
                            Piece::new(PieceType::Bishop, color.clone(), 5),
                            Piece::new(PieceType::Queen, color.clone(), 3),
                            Piece::new(PieceType::King, color.clone(), 4),
                        ]
                        .into_iter(),
                    )
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        Ok(Self { pieces })
    }

    pub fn find_own_piece_at_mut(
        &mut self,
        position: &Position,
        color: &PieceColor,
    ) -> Option<&mut Piece> {
        self.pieces
            .iter_mut()
            .find(move |piece| piece.position == *position && piece.color == *color)
    }

    pub fn find_king(&self, color: PieceColor) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|piece| piece.color == color && piece.piece_type == PieceType::King)
    }

    pub fn remove_piece(&mut self, position: &Position, color: &PieceColor) -> Option<Piece> {
        let position = self.pieces.iter().position(move |current_piece| {
            current_piece.position == *position && current_piece.color == *color
        })?;
        Some(self.pieces.swap_remove(position))
    }

    pub fn is_path_clear(&self, from: &Position, to: &Position) -> bool {
        let from_row = from.row;
        let to_row = to.row;
        let from_col = from.column;
        let to_col = to.column;
        // Movement in the same column
        if from.column == to.column {
            for piece in self.pieces.iter() {
                if piece.position.column != from.column {
                    continue;
                }
                let row = piece.position.row;
                if (row > from_row && row < to_row) || (row > to_row && row < from_row) {
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
                if (col > from_col && col < to_col) || (col > to_col && col < from_col) {
                    return false;
                }
            }
        };
        // Diagonal
        let diff_col = (from_col - to_col).abs();
        let diff_row = (from_row - to_row).abs();
        if diff_col == diff_row {
            for i in 1..diff_col {
                let pos = if from_col < to_col && from_row < to_row {
                    (from_col + i, from_row + i)
                } else if from_col < to_col && from_row > to_row {
                    (from_col + i, from_row - i)
                } else if from_col > to_col && from_row < to_row {
                    (from_col - i, from_row + i)
                } else {
                    (from_col - i, from_row - i)
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
        player_color: &PieceColor,
        from: &Position,
        to: &Position,
    ) -> anyhow::Result<Option<Piece>> {
        if !self.is_path_clear(from, to) {
            bail!("The path is currently occupied");
        }
        let piece = self.find_own_piece_at_mut(from, player_color);
        let piece = piece.ok_or(anyhow!("You don't have a piece at this position!"))?;
        piece.move_piece_to(to).await?;
        Ok(self.remove_piece(to, &player_color.invert()))
    }
}

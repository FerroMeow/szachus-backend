use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::routes::game::piece_color::PieceColor;

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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PieceColor,
    pub position: Position,
    pub times_moved: u8,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: PieceColor, column: i8) -> Piece {
        let row = match (&piece_type, &color) {
            (PieceType::Pawn, PieceColor::White) => 1,
            (PieceType::Pawn, PieceColor::Black) => 6,
            (_, PieceColor::White) => 0,
            (_, PieceColor::Black) => 7,
        };
        Piece {
            piece_type,
            color,
            position: Position::new(column, row),
            times_moved: 0,
        }
    }

    pub async fn move_piece_to(&mut self, new_position: Position) -> anyhow::Result<()> {
        let position_difference = new_position - self.position;
        match self.piece_type {
            PieceType::Pawn => {
                self.pawn_move(new_position, position_difference).await?;
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
        self.position = new_position;
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

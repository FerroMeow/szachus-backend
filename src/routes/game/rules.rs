use std::sync::Arc;

use anyhow::bail;
use futures::lock::Mutex;
use rust_decimal::prelude::Zero;
use serde::{Deserialize, Serialize};

use super::ArcMut;

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

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Row(u8);

impl Row {
    pub fn new(row: i8) -> anyhow::Result<Self> {
        if !(0..=7).contains(&row) {
            bail!("Row must be between 0 and 7");
        }
        Ok(Self(row as u8))
    }
}

impl std::ops::Add for Row {
    type Output = anyhow::Result<Row>;

    fn add(self, rhs: Self) -> Self::Output {
        let new_value = self.0 + rhs.0;
        if new_value > 7 {
            bail!("Row must be between 0 and 7");
        };
        Ok(Row(new_value))
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Column(u8);

impl Column {
    pub fn new(column: i8) -> anyhow::Result<Self> {
        if !(0..=7).contains(&column) {
            bail!("Column must be between 0 and 7");
        }
        Ok(Self(column as u8))
    }
}

impl std::ops::Add for Column {
    type Output = anyhow::Result<Column>;

    fn add(self, rhs: Self) -> Self::Output {
        let new_value = self.0 + rhs.0;
        if new_value > 7 {
            bail!("Column must be between 0 and 7");
        };
        Ok(Column(new_value))
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Position {
    row: Row,
    column: Column,
}

impl Position {
    pub fn new(row: i8, column: i8) -> anyhow::Result<Self> {
        Ok(Position {
            row: Row::new(row)?,
            column: Column::new(column)?,
        })
    }
}

impl std::ops::Sub for Position {
    type Output = (i8, i8);

    fn sub(self, rhs: Self) -> Self::Output {
        (
            (self.row.0 as i8 - rhs.row.0 as i8).abs(),
            (self.column.0 as i8 - rhs.column.0 as i8).abs(),
        )
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
            position: Position::new(row, column)?,
            times_moved: 0,
        })
    }

    pub async fn move_piece_to(
        &mut self,
        game: Arc<Mutex<ChessBoard>>,
        new_position: &Position,
    ) -> anyhow::Result<()> {
        let position_difference = new_position.clone() - self.position.clone();
        if let PieceColor::Black = self.color {
            // position_difference.0 *= -1;
        };
        match self.piece_type {
            PieceType::Pawn => {
                self.pawn_move(new_position.clone(), position_difference, game)
                    .await?;
                match self.color {
                    PieceColor::White if new_position.row.0 == 7 => {
                        self.piece_type = PieceType::Queen;
                    }
                    PieceColor::Black if new_position.row.0 == 0 => {
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
                let _ = game.lock().await.remove_piece_at(new_position, &self.color);
                self.position = new_position.clone();
            }
            PieceType::King => {
                if !(position_difference.0.abs() <= 1 && position_difference.1.abs() <= 1) {
                    bail!("Incorrect King move!");
                }
                let _ = game
                    .clone()
                    .lock()
                    .await
                    .remove_piece_at(new_position, &self.color);
                self.position = new_position.clone();
            }
            PieceType::Rook => {
                self.rook_move(new_position.clone(), position_difference, game.clone())
                    .await?;
                let _ = game
                    .clone()
                    .lock()
                    .await
                    .remove_piece_at(new_position, &self.color);
                self.position = new_position.clone();
            }
            PieceType::Bishop => {
                self.bishop_move(new_position.clone(), position_difference, game.clone())
                    .await?;
                let _ = game
                    .clone()
                    .lock()
                    .await
                    .remove_piece_at(new_position, &self.color);
                self.position = new_position.clone();
            }
            PieceType::Queen => {
                let move_successful = [
                    self.rook_move(new_position.clone(), position_difference, game.clone())
                        .await,
                    self.bishop_move(new_position.clone(), position_difference, game.clone())
                        .await,
                ]
                .iter()
                .any(|result| result.is_ok());
                if !move_successful {
                    bail!("Incorrect queen move");
                }
                let _ = game
                    .clone()
                    .lock()
                    .await
                    .remove_piece_at(new_position, &self.color);
                self.position = new_position.clone();
            }
        };
        self.times_moved += 1;
        Ok(())
    }

    async fn rook_move(
        &mut self,
        new_position: Position,
        position_difference: (i8, i8),
        game: Arc<Mutex<ChessBoard>>,
    ) -> Result<(), anyhow::Error> {
        let (row_mod, col_mod) = match position_difference {
            (row, col) if (row.is_negative() && col.is_zero()) => (-1, 0),
            (row, col) if (row.is_positive() && col.is_zero()) => (1, 0),
            (row, col) if (row.is_zero() && col.is_negative()) => (0, -1),
            (row, col) if (row.is_zero() && col.is_positive()) => (0, 1),
            _ => bail!("Incorrect rook move"),
        };
        let mut current_position = self.position.clone();
        loop {
            current_position = Position::new(
                current_position.row.0 as i8 + row_mod,
                self.position.column.0 as i8 + col_mod,
            )?;
            if current_position == new_position {
                return Ok(());
            }
            if game
                .clone()
                .lock()
                .await
                .find_piece_at(&current_position)
                .is_some()
            {
                bail!("Cannot move over a tile");
            }
        }
    }

    async fn bishop_move(
        &mut self,
        new_position: Position,
        position_difference: (i8, i8),
        game: Arc<Mutex<ChessBoard>>,
    ) -> Result<(), anyhow::Error> {
        if position_difference.0 == 0
            || position_difference.1 == 0
            || position_difference.0.abs() != position_difference.1.abs()
        {
            bail!("Incorrect bishop move");
        };
        let (row_mod, col_mod) = match position_difference {
            (row, col) if (row.is_negative() && col.is_negative()) => (-1, -1),
            (row, col) if (row.is_negative() && col.is_positive()) => (-1, 1),
            (row, col) if (row.is_positive() && col.is_positive()) => (1, 1),
            (row, col) if (row.is_positive() && col.is_negative()) => (1, -1),
            _ => bail!("Incorrect bishop move"),
        };
        let mut current_position = self.position.clone();
        loop {
            current_position = Position::new(
                current_position.row.0 as i8 + row_mod,
                self.position.column.0 as i8 + col_mod,
            )?;
            if current_position == new_position {
                return Ok(());
            }
            if game
                .clone()
                .lock()
                .await
                .find_piece_at(&current_position)
                .is_some()
            {
                bail!("Cannot move over a tile");
            }
        }
    }

    async fn pawn_move(
        &mut self,
        new_position: Position,
        position_difference: (i8, i8),
        game: Arc<Mutex<ChessBoard>>,
    ) -> Result<(), anyhow::Error> {
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
            if game
                .clone()
                .lock()
                .await
                .find_piece_at(&new_position)
                .is_some()
            {
                bail!("Cannot move to the new position, there is already a piece there.");
            }
            self.position = new_position;
            return Ok(());
        };
        if position_difference.1.abs() == 1
            && game
                .clone()
                .lock()
                .await
                .find_piece_at(&new_position)
                .is_some()
        {
            game.clone()
                .lock()
                .await
                .remove_piece_at(&new_position, &self.color)?;
            self.position = new_position;
            return Ok(());
        }
        bail!("Movement unreachable by any means");
    }
}

#[derive(Clone, Debug)]
pub struct ChessBoard {
    pub pieces: Vec<ArcMut<Piece>>,
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
            .map(|p| Arc::new(Mutex::new(p)))
            .collect::<Vec<_>>();
        Ok(Self { pieces })
    }

    pub fn find_piece_at(&self, position: &Position) -> Option<&ArcMut<Piece>> {
        self.pieces.iter().find(move |piece| {
            piece
                .try_lock()
                .map(|piece| piece.position == *position)
                .unwrap_or(false)
        })
    }

    pub fn find_own_piece_at(
        &mut self,
        position: &Position,
        color: PieceColor,
    ) -> Option<ArcMut<Piece>> {
        self.pieces
            .iter()
            .find(move |piece| {
                piece.try_lock().unwrap().position == *position
                    && piece.try_lock().unwrap().color == color
            })
            .cloned()
    }

    pub fn find_king(&mut self, color: PieceColor) -> Option<ArcMut<Piece>> {
        self.pieces
            .iter()
            .find(|piece| {
                piece.try_lock().unwrap().color == color
                    && piece
                        .try_lock()
                        .map(|piece| piece.piece_type == PieceType::King)
                        .unwrap_or_default()
            })
            .cloned()
    }

    pub fn remove_piece_at(
        &mut self,
        position: &Position,
        color: &PieceColor,
    ) -> anyhow::Result<Option<ArcMut<Piece>>> {
        let Some(position) = self.pieces.iter().position(move |piece| {
            piece
                .try_lock()
                .map_or(false, |piece| piece.position == *position)
        }) else {
            bail!("piece not found at the new position");
        };
        if let Some(pawn) = self.pieces.get(position) {
            if pawn.try_lock().unwrap().color == *color {
                return Ok(None);
            }
        } else {
            bail!("Piece not found!");
        }
        Ok(Some(self.pieces.swap_remove(position)))
    }
}

use anyhow::bail;
use rust_decimal::prelude::Zero;

pub enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn,
}

#[derive(PartialEq)]
enum PieceColor {
    White,
    Black,
}

#[derive(PartialEq, Clone)]
struct Row(u8);

impl Row {
    pub fn new(row: i8) -> anyhow::Result<Self> {
        if row < 0 || row > 7 {
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

#[derive(PartialEq, Clone)]
struct Column(u8);

impl Column {
    pub fn new(column: i8) -> anyhow::Result<Self> {
        if column < 0 || column > 7 {
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

#[derive(PartialEq, Clone)]
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
            self.row.0 as i8 - rhs.row.0 as i8,
            self.column.0 as i8 - rhs.column.0 as i8,
        )
    }
}

pub struct Piece {
    piece_type: PieceType,
    color: PieceColor,
    position: Position,
    times_moved: u8,
}

impl Piece {
    pub fn move_piece_to(
        &mut self,
        game: &mut ChessGame,
        new_position: Position,
    ) -> anyhow::Result<()> {
        let mut position_difference = new_position.clone() - self.position.clone();
        if let PieceColor::Black = self.color {
            position_difference.0 *= -1;
        };
        match self.piece_type {
            PieceType::Pawn => {
                self.pawn_move(new_position.clone(), position_difference, game)?;
                match self.color {
                    PieceColor::White if new_position.row.0 == 7 => {
                        self.piece_type = PieceType::Queen;
                    }
                    PieceColor::Black if new_position.row.0 == 0 => {
                        self.piece_type = PieceType::Queen;
                    }
                    _ => (),
                };
                Ok(())
            }
            PieceType::Knight => {
                if !((position_difference.0.abs() == 1 && position_difference.1.abs() == 2)
                    || (position_difference.0.abs() == 2 && position_difference.1.abs() == 1))
                {
                    bail!("Incorrect knight move!");
                }
                let _ = game.remove_piece_at(&new_position, &self.color);
                self.position = new_position;
                Ok(())
            }
            PieceType::King => {
                if !(position_difference.0.abs() <= 1 && position_difference.1.abs() <= 1) {
                    bail!("Incorrect King move!");
                }
                let _ = game.remove_piece_at(&new_position, &self.color);
                self.position = new_position;
                Ok(())
            }
            PieceType::Rook => {
                self.rook_move(new_position.clone(), position_difference, game)?;
                let _ = game.remove_piece_at(&new_position, &self.color);
                self.position = new_position;
                Ok(())
            }
            PieceType::Bishop => {
                self.bishop_move(new_position.clone(), position_difference, game)?;
                let _ = game.remove_piece_at(&new_position, &self.color);
                self.position = new_position;
                Ok(())
            }
            PieceType::Queen => {
                let move_successful = [
                    self.rook_move(new_position.clone(), position_difference, game),
                    self.bishop_move(new_position.clone(), position_difference, game),
                ]
                .iter()
                .any(|result| result.is_ok());
                if !move_successful {
                    bail!("Incorrect queen move");
                }
                let _ = game.remove_piece_at(&new_position, &self.color);
                self.position = new_position;
                Ok(())
            }
            _ => bail!("Unrecognized piece type"),
        }
    }

    fn rook_move(
        &mut self,
        new_position: Position,
        position_difference: (i8, i8),
        game: &mut ChessGame,
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
            if game.piece_at(&current_position).is_some() {
                bail!("Cannot move over a tile");
            }
        }
    }

    fn bishop_move(
        &mut self,
        new_position: Position,
        position_difference: (i8, i8),
        game: &mut ChessGame,
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
            if game.piece_at(&current_position).is_some() {
                bail!("Cannot move over a tile");
            }
        }
    }

    fn pawn_move(
        &mut self,
        new_position: Position,
        position_difference: (i8, i8),
        game: &mut ChessGame,
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
            if game.piece_at(&new_position).is_some() {
                bail!("Cannot move to the new position, there is already a piece there.");
            }
            self.position = new_position;
            return Ok(());
        };
        if position_difference.1.abs() == 1 && game.piece_at(&new_position).is_some() {
            game.remove_piece_at(&new_position, &self.color)?;
            self.position = new_position;
            return Ok(());
        }
        bail!("Movement unreachable by any means");
    }
}

pub struct ChessGame {
    pawns: Vec<Piece>,
}

impl ChessGame {
    pub fn piece_at(&self, position: &Position) -> Option<&Piece> {
        self.pawns
            .iter()
            .find(move |piece| piece.position == *position)
    }
    pub fn enemy_piece_at(&self, position: &Position, color: PieceColor) -> Option<&Piece> {
        self.pawns
            .iter()
            .find(move |piece| piece.position == *position && piece.color != color)
    }

    pub fn remove_piece_at(
        &mut self,
        position: &Position,
        color: &PieceColor,
    ) -> anyhow::Result<Option<Piece>> {
        let Some(position) = self
            .pawns
            .iter()
            .position(move |piece| piece.position == *position)
        else {
            bail!("piece not found at the new position");
        };
        if let Some(pawn) = self.pawns.get(position) {
            if pawn.color == *color {
                return Ok(None);
            }
        } else {
            bail!("Piece not found!");
        }
        Ok(Some(self.pawns.swap_remove(position)))
    }
}

use serde::{Deserialize, Serialize};

use crate::routes::game::piece_color::PieceColor;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Position {
    pub column: i8,
    pub row: i8,
}

impl Position {
    pub fn new(column: i8, row: i8) -> Self {
        Position { row, column }
    }

    pub fn invert(&self) -> Self {
        Position {
            row: 7 - self.row,
            column: 7 - self.column,
        }
    }

    pub fn maybe_invert(&self, color: PieceColor) -> Self {
        match color {
            PieceColor::Black => self.invert(),
            PieceColor::White => *self,
        }
    }
}

impl std::ops::Sub for Position {
    type Output = (i8, i8);

    fn sub(self, rhs: Self) -> Self::Output {
        ((self.row - rhs.row).abs(), (self.column - rhs.column).abs())
    }
}

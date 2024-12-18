use std::fmt;

use serde::{Deserialize, Serialize};

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
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let col = match self.column {
            0 => "A",
            1 => "B",
            2 => "C",
            3 => "D",
            4 => "E",
            5 => "F",
            6 => "G",
            7 => "H",
            _ => {
                return Err(fmt::Error);
            }
        };
        let row = match self.row {
            0 => "1",
            1 => "2",
            2 => "3",
            3 => "4",
            4 => "5",
            5 => "6",
            6 => "7",
            7 => "8",
            _ => {
                return Err(fmt::Error);
            }
        };
        write!(f, "{col}{row}")
    }
}

impl std::ops::Sub for Position {
    type Output = (i8, i8);

    fn sub(self, rhs: Self) -> Self::Output {
        ((self.row - rhs.row).abs(), (self.column - rhs.column).abs())
    }
}

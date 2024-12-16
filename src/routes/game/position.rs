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
}

impl std::ops::Sub for Position {
    type Output = (i8, i8);

    fn sub(self, rhs: Self) -> Self::Output {
        ((self.row - rhs.row).abs(), (self.column - rhs.column).abs())
    }
}

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, Debug)]
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

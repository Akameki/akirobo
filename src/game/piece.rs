//! Information about pieces, including initial spawn location/orientation, and all of their spins/kicks.
//! As a broad overview, the current piece is stored as indices of a 4x4 grid, along with its position in the game matrix.

use crate::botris::types::Piece;
use super::matrix::{Matrix, EMPTY_BOARD};

impl Piece {
    pub fn rotations(&self) -> [PieceCoords; 4] {
        match self {
            Piece::I => [[(2, 0), (2, 1), (2, 2), (2, 3)],
                         [(0, 2), (1, 2), (2, 2), (3, 2)],
                         [(1, 0), (1, 1), (1, 2), (1, 3)],
                         [(0, 1), (1, 1), (2, 1), (3, 1)]],
            Piece::J => [[(2, 0), (2, 1), (2, 2), (3, 0)],
                         [(1, 1), (2, 1), (3, 1), (3, 2)],
                         [(2, 0), (2, 1), (2, 2), (1, 2)],
                         [(1, 0), (1, 1), (2, 1), (3, 1)]],
            Piece::L => [[(2, 0), (2, 1), (2, 2), (3, 2)],
                         [(1, 1), (1, 2), (2, 1), (3, 1)],
                         [(1, 0), (2, 0), (2, 1), (2, 2)],
                         [(1, 1), (2, 1), (3, 0), (3, 1)]],
            Piece::O => [[(2, 1), (2, 2), (3, 1), (3, 2)]; 4],
            Piece::S => [[(2, 0), (2, 1), (3, 1), (3, 2)],
                         [(1, 2), (2, 1), (2, 2), (3, 1)],
                         [(1, 0), (1, 1), (2, 1), (2, 2)],
                         [(1, 1), (2, 0), (2, 1), (3, 0)]],
            Piece::T => [[(2, 0), (2, 1), (2, 2), (3, 1)],
                         [(1, 1), (2, 1), (3, 1), (2, 2)],
                         [(2, 0), (1, 1), (2, 1), (2, 2)],
                         [(1, 1), (2, 0), (2, 1), (3, 1)]],
            Piece::Z => [[(2, 1), (2, 2), (3, 0), (3, 1)],
                         [(1, 1), (2, 1), (2, 2), (3, 2)],
                         [(1, 1), (1, 2), (2, 0), (2, 1)],
                         [(1, 0), (2, 0), (2, 1), (3, 1)]],
        }
    }
}

pub type PieceCoords = [(i8, i8); 4]; // can be use relative or absolute

#[derive(Debug, Clone)]
pub struct CurrentPiece {
    pub piece: Piece,
    pub y: i8,
    pub x: i8,
    pub rotation: usize,
}

impl CurrentPiece {
    pub fn new(piece: Piece) -> Self {
        CurrentPiece {
            piece,
            y: 17,
            x: 3,
            rotation: 0,
        }
    }

    pub fn relative(&self) -> PieceCoords {
        self.piece.rotations()[self.rotation]
    }

    // return the absolute positions of the piece in the game matrix
    pub fn absolute(&self) -> PieceCoords {
        let mut result = [(self.y, self.x); 4];
        let relative = self.relative();
        for i in 0..4 {
            result[i].0 += relative[i].0;
            result[i].1 += relative[i].1;
        }
        result
    }
}



//            Piece::I => {
//     [
//         0b0000,
//         0b1111,
//         0b0000,
//         0b0000,
//     ]
// },
// Piece::O => {
//     [
//         0b11,
//         0b11,
//     ]
// },
// Piece::J => {
//     [
//         0b100,
//         0b111,
//         0b000,
//     ]
// },
// Piece::L => {
//     [
//         0b001,
//         0b111,
//         0b000,
//     ]
// },
// Piece::S => {
//     [
//         0b011,
//         0b110,
//         0b000,
//     ]
// },
// Piece::Z => {
//     [
//         0b110,
//         0b011,
//         0b000,
//     ]
// },
// Piece::T => {
//     [
//         0b010,
//         0b111,
//         0b000,
//     ]
// },
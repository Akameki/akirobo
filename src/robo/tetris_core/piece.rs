//! Information about pieces, including initial spawn location/orientation, and all of their spins/kicks.
//! As a broad overview, the current piece is stored as indices of a 4x4 grid, along with its position in the game matrix.

use crate::botris::types::{Command, Piece};

impl Piece {
    pub fn rotations(&self) -> [PieceCoords; 4] {
        match self {
            Piece::I => [
                [(2, 0), (2, 1), (2, 2), (2, 3)],
                [(0, 2), (1, 2), (2, 2), (3, 2)],
                [(1, 0), (1, 1), (1, 2), (1, 3)],
                [(0, 1), (1, 1), (2, 1), (3, 1)],
            ],
            Piece::J => [
                [(2, 0), (2, 1), (2, 2), (3, 0)],
                [(1, 1), (2, 1), (3, 1), (3, 2)],
                [(2, 0), (2, 1), (2, 2), (1, 2)],
                [(1, 0), (1, 1), (2, 1), (3, 1)],
            ],
            Piece::L => [
                [(2, 0), (2, 1), (2, 2), (3, 2)],
                [(1, 1), (1, 2), (2, 1), (3, 1)],
                [(1, 0), (2, 0), (2, 1), (2, 2)],
                [(1, 1), (2, 1), (3, 0), (3, 1)],
            ],
            Piece::O => [[(2, 1), (2, 2), (3, 1), (3, 2)]; 4],
            Piece::S => [
                [(2, 0), (2, 1), (3, 1), (3, 2)],
                [(1, 2), (2, 1), (2, 2), (3, 1)],
                [(1, 0), (1, 1), (2, 1), (2, 2)],
                [(1, 1), (2, 0), (2, 1), (3, 0)],
            ],
            Piece::T => [
                [(2, 0), (2, 1), (2, 2), (3, 1)],
                [(1, 1), (2, 1), (3, 1), (2, 2)],
                [(2, 0), (1, 1), (2, 1), (2, 2)],
                [(1, 1), (2, 0), (2, 1), (3, 1)],
            ],
            Piece::Z => [
                [(2, 1), (2, 2), (3, 0), (3, 1)],
                [(1, 1), (2, 1), (2, 2), (3, 2)],
                [(1, 1), (1, 2), (2, 0), (2, 1)],
                [(1, 0), (2, 0), (2, 1), (3, 1)],
            ],
        }
    }

    /// Returns [dx, dy] kicks to try into a target rotation.
    pub fn kicks(&self, command: Command) -> [[[i8; 2]; 5]; 4] {
        match self {
            Piece::I => match command {
                Command::RotateCcw => [
                    [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]], // 1-0
                    [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]], // 2-1
                    [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]], // 3-2
                    [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]], // 0-3
                ],
                Command::RotateCw => [
                    [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]], // 3-0
                    [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]], // 0-1
                    [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]], // 1-2
                    [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]], // 2-3
                ],
                _ => panic!(),
            },
            Piece::O => match command {
                Command::RotateCcw => [
                    [[-1, 1], [-2, 1], [-1, 2], [0, 1], [0, 0]],     // 1-0
                    [[1, 1], [2, 1], [1, 2], [0, 1], [0, 0]],        // 2-1
                    [[1, -1], [2, -1], [1, -2], [0, -1], [0, 0]],    // 3-2
                    [[-1, -1], [-2, -1], [-1, -2], [0, -1], [0, 0]], // 0-3
                ],
                Command::RotateCw => [
                    [[1, 1], [2, 1], [1, 2], [0, 1], [0, 0]],        // 3-0
                    [[1, -1], [2, -1], [1, -2], [0, -1], [0, 0]],    // 0-1
                    [[-1, -1], [-2, -1], [-1, -2], [0, -1], [0, 0]], // 1-2
                    [[-1, 1], [-2, 1], [-1, 2], [0, 1], [0, 0]],     // 2-3
                ],
                _ => panic!(),
            },
            _ => match command {
                Command::RotateCcw => [
                    [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],     // 1-0
                    [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]], // 2-1
                    [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],  // 3-2
                    [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],    // 0-3
                ],
                Command::RotateCw => [
                    [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],  // 3-0
                    [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]], // 0-1
                    [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],     // 1-2
                    [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],    // 2-3
                ],
                _ => panic!(),
            },
        }
    }
}

pub type PieceCoords = [(i8, i8); 4]; // can be use relative or absolute

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FallingPiece {
    pub rotation: usize,
    pub piece: Piece,
    pub y: i8,
    pub x: i8,
}

impl FallingPiece {
    pub fn new(piece: Piece) -> Self {
        FallingPiece { piece, y: 17, x: 3, rotation: 0 }
    }

    fn relative(&self) -> PieceCoords {
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

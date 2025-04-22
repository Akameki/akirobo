//! Information about pieces, including initial spawn location/orientation, and all of their spins/kicks.
//! As a broad overview, the current piece is stored as indices of a 4x4 grid, along with its position in the game matrix.

use std::hash::Hash;

use crate::botris::types::{Command, Piece};

impl Piece {
    fn rotations(&self) -> [PieceCoords; 4] {
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

/// absolute coordinates. signed to allow kick
pub type PieceCoords = [(i8, i8); 4];

#[derive(Debug, Clone, Copy, Eq)]
pub struct FallingPiece {
    pub piece: Piece,
    pub rotation: u8,
    /// may be out of bounds! check wallkicks after rotating the piece.
    pub coords: PieceCoords,
}

impl FallingPiece {
    pub fn new(piece: Piece) -> Self {
        FallingPiece {
            piece,
            rotation: 0,
            coords: match piece {
                Piece::I => [(19, 3), (19, 4), (19, 5), (19, 6)],
                Piece::O => [(19, 4), (19, 5), (20, 4), (20, 5)],
                Piece::J => [(19, 3), (19, 4), (19, 5), (20, 3)],
                Piece::L => [(19, 3), (19, 4), (19, 5), (20, 5)],
                Piece::S => [(19, 3), (19, 4), (20, 4), (20, 5)],
                Piece::Z => [(19, 4), (19, 5), (20, 3), (20, 4)],
                Piece::T => [(19, 3), (19, 4), (19, 5), (20, 4)],
            },
        }
    }

    pub fn rotate(&mut self, rotation: Command) {
        // TODO: precalculate all diffs
        let new_rotation = (self.rotation
            + match rotation {
                Command::RotateCcw => 3,
                Command::RotateCw => 1,
                _ => panic!(),
            })
            % 4;
        let rotations = self.piece.rotations();
        let old_rel_coords = rotations[self.rotation as usize];
        let new_rel_coords = rotations[new_rotation as usize];
        for i in 0..4 {
            self.coords[i].0 += new_rel_coords[i].0 - old_rel_coords[i].0;
            self.coords[i].1 += new_rel_coords[i].1 - old_rel_coords[i].1;
        }
        self.rotation = new_rotation;
    }

    pub fn shift(&mut self, dy: i8, dx: i8) {
        for i in 0..4 {
            self.coords[i].0 += dy;
            self.coords[i].1 += dx;
        }
    }
}

impl Hash for FallingPiece {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.piece.hash(state);
        self.rotation.hash(state);
        self.coords[0].hash(state);
    }
}
impl PartialEq for FallingPiece {
    fn eq(&self, other: &Self) -> bool {
        self.piece == other.piece && self.rotation == other.rotation && self.coords[0] == other.coords[0]
    }
}
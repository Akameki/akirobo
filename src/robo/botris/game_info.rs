use super::types::{ClearName, ClearName::*};

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 30;

impl ClearName {
    pub fn attack(self) -> u32 {
        match self {
            Single => 0,
            Double => 1,
            Triple => 2,
            Quad => 4,
            ASS => 4,
            ASD => 4,
            AST => 6,
            PC => 10,
        }
    }
}

pub const B2B_ATTACK: u32 = 1;

pub const COMBO_TABLE: [u8; 25] =
    [0, 0, 1, 1, 1, 2, 2, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];

use super::piece::{FallingPiece, PieceCoords};
use crate::botris::{self, types::Command};

// index 0 is the bottom of the board
// pub type Board = [[bool; 10]; BOARD_HEIGHT];
// pub const EMPTY_BOARD: Board = [[false; 10]; BOARD_HEIGHT];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BitBoard {
    cols: [u32; 10],
}
pub const BITBOARD_HEIGHT: usize = 32;

pub const EMPTY_BOARD: BitBoard = BitBoard { cols: [0; 10] };

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BoardData {
    pub b2b: bool,
    pub incoming: [u32; 8],
    pub combo: u32,
    pub cummulative_attack: u32,
    pub simulated_garbage: u32,
}

impl BitBoard {
    pub fn from_strs(strs: &[&str]) -> BitBoard {
        debug_assert_eq!(strs[0].len(), 20);
        let mut board = EMPTY_BOARD;
        for (y, row) in strs.iter().rev().enumerate() {
            for (x, cell) in row.chars().step_by(2).enumerate() {
                board.set(y, x, cell != ' ');
            }
        }
        board
    }
    pub fn from_thin_strs(strs: &[&str]) -> BitBoard {
        debug_assert_eq!(strs[0].len(), 10);
        let mut board = EMPTY_BOARD;
        for (y, row) in strs.iter().rev().enumerate() {
            for (x, cell) in row.chars().enumerate() {
                board.set(y, x, cell != ' ');
            }
        }
        board
    }

    pub fn at(&self, row: usize, col: usize) -> bool {
        self.cols[col] & (1 << row) != 0
    }
    pub fn set(&mut self, row: usize, col: usize, val: bool) {
        if val {
            self.cols[col] |= 1 << row;
        } else {
            self.cols[col] &= !(1 << row);
        }
    }
    pub fn column_height(&self, col: usize) -> usize {
        BITBOARD_HEIGHT - self.cols[col].leading_zeros() as usize
    }
    pub fn stack_height(&self) -> usize {
        BITBOARD_HEIGHT - self.cols.iter().map(|&col| col.leading_zeros()).min().unwrap() as usize
    }
    pub fn collides(&self, tentative_piece: &FallingPiece) -> bool {
        tentative_piece.absolute().iter().any(|&(y, x)| {
            x < 0
                || y < 0
                || x >= 10
                || y >= BITBOARD_HEIGHT as i8
                || self.at(y as usize, x as usize)
        })
    }
    pub fn print_board(&self, piece: Option<PieceCoords>) {
        Self::print_rows(&[(self, piece)], 1);
    }
    pub fn print_rows(boards: &[(&Self, Option<PieceCoords>)], row_size: usize) {
        println!("{}", ">~~~~~~~~~~~~~~~~~~~~<".repeat(std::cmp::min(boards.len(), row_size)));
        for chunk in boards.chunks(row_size) {
            let highest_row = chunk
                .iter()
                .map(|(board, piece_coords)| {
                    let piece_height = if let Some(coords) = piece_coords {
                        coords.iter().map(|&(y, _)| y).max().unwrap() as usize
                    } else {
                        0
                    };
                    std::cmp::max(board.stack_height(), piece_height)
                })
                .max()
                .unwrap();
            let rows_to_print = std::cmp::min(highest_row + 1, BITBOARD_HEIGHT);
            for row in (0..rows_to_print).rev() {
                for (board, piece_coords) in chunk {
                    print!("\"");
                    for col in 0..10 {
                        if piece_coords
                            .is_some_and(|coords| coords.contains(&(row as i8, col as i8)))
                        {
                            print!("██");
                        } else if board.at(row, col) {
                            print!("[]");
                        } else {
                            print!("  ");
                        }
                    }
                    print!("\"");
                }
                println!();
            }
            println!("{}", ">~~~~~~~~~~~~~~~~~~~~<".repeat(chunk.len()));
        }
    }
    /// returns None if the command is impossible or does nothing.
    pub fn try_command(
        &self,
        falling_piece: &FallingPiece,
        command: Command,
    ) -> Option<FallingPiece> {
        use Command::*;
        let mut tentative_piece = *falling_piece;
        match command {
            MoveLeft => tentative_piece.x -= 1,
            MoveRight => tentative_piece.x += 1,
            Drop => tentative_piece.y -= 1,
            SonicDrop => {
                while !self.collides(&tentative_piece) {
                    tentative_piece.y -= 1;
                }
                tentative_piece.y += 1;
            }
            RotateCw | RotateCcw => {
                tentative_piece.rotation += if command == RotateCw { 1 } else { 3 };
                tentative_piece.rotation %= 4;

                // go through kicks in kicktable
                for [x_kick, y_kick] in
                    tentative_piece.piece.kicks(command)[tentative_piece.rotation]
                {
                    let rotated = FallingPiece {
                        x: tentative_piece.x + x_kick,
                        y: tentative_piece.y + y_kick,
                        rotation: tentative_piece.rotation,
                        piece: tentative_piece.piece,
                    };
                    if !self.collides(&rotated) {
                        return Some(rotated);
                    }
                }
                return None;
            }
            _ => panic!(),
        }

        if self.collides(&tentative_piece) || tentative_piece == *falling_piece {
            None
        } else {
            Some(tentative_piece)
        }
    }
    pub fn try_commands(
        &self,
        falling_piece: &FallingPiece,
        commands: &[Command],
    ) -> Option<FallingPiece> {
        commands
            .iter()
            .try_fold(*falling_piece, |falling, &command| self.try_command(&falling, command))
    }

    pub fn force_sonic_drop(&self, falling_piece: &FallingPiece) -> FallingPiece {
        let mut tentative_piece = *falling_piece;
        while !self.collides(&tentative_piece) {
            tentative_piece.y -= 1;
        }
        tentative_piece.y += 1;
        tentative_piece
    }

    /* lock */

    pub fn hard_drop(&self, all_spin: bool, data: BoardData) -> (BitBoard, BoardData) {
        let mut new_board = *self;
        let mut new_data = data;

        // clear lines
        let mut cleared_lines = 0;
        for row in data.simulated_garbage as usize..self.stack_height() {
            if new_board.cols.iter().all(|&col| col & (1 << row) != 0) {
                cleared_lines += 1;
                for col in &mut new_board.cols {
                    let below_clear_mask = (1 << row) - 1;
                    let above_clear_mask = !((below_clear_mask << 1) + 1);
                    let below_clear = *col & below_clear_mask;
                    *col = (*col & above_clear_mask) >> 1 | below_clear;
                }
            }
        }

        if cleared_lines > 0 {
            new_data.combo += 1;
            new_data.b2b = all_spin || cleared_lines == 4;
            use crate::botris::{game_info::*, types::ClearName::*};
            if all_spin {
                new_data.cummulative_attack += match cleared_lines {
                    1 => ASS.attack(),
                    2 => ASD.attack(),
                    3 => AST.attack(),
                    _ => panic!(),
                };
            } else {
                new_data.cummulative_attack += match cleared_lines {
                    1 => Single.attack(),
                    2 => Double.attack(),
                    3 => Triple.attack(),
                    4 => Quad.attack(),
                    _ => panic!(),
                };
            }
            new_data.cummulative_attack += COMBO_TABLE[new_data.combo as usize] as u32;
            if new_board.cols.iter().all(|&x| x == 0) {
                new_data.cummulative_attack += PC.attack();
            }
            if data.b2b && new_data.b2b {
                new_data.cummulative_attack += B2B_ATTACK;
            }
            new_data.incoming[0] += new_data.incoming[1];
        } else {
            new_data.combo = 0;
            let new_garbage_lines = new_data.incoming[0] as usize;
            if new_garbage_lines >= BITBOARD_HEIGHT {
                new_board.cols = [u32::MAX; 10];
            } else if new_garbage_lines > 0 {
                for col in &mut new_board.cols {
                    *col = (*col << new_garbage_lines) | ((1 << new_garbage_lines) - 1);
                }
            }
            new_data.simulated_garbage += new_garbage_lines as u32;
            new_data.incoming[0] = 0;
        }
        // incoming[0] properly updated. now shift everything else over by 1
        new_data.incoming.copy_within(2.., 1);

        (new_board, new_data)
    }
}

// converts board as returned by Botris API to the internal board representation
// TODO: move me
pub fn to_board(board: &botris::types::Board) -> BitBoard {
    let mut new_board = EMPTY_BOARD;
    for (y, row) in board.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            new_board.set(y, x, cell.is_some());
        }
    }
    new_board
}

use super::piece::{FallingPiece, PieceCoords};
use crate::botris::{self, types::Command};

// index 0 is the bottom of the board
// pub type Board = [[bool; 10]; BOARD_HEIGHT];
// pub const EMPTY_BOARD: Board = [[false; 10]; BOARD_HEIGHT];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BitBoard {
    pub cols: [u32; 10],
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

    #[inline]
    pub fn at(&self, row: usize, col: usize) -> bool {
        self.cols[col] & (1 << row) != 0
    }
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, val: bool) {
        if val {
            self.cols[col] |= 1 << row;
        } else {
            self.cols[col] &= !(1 << row);
        }
    }
    #[inline]
    pub fn column_height(&self, col: usize) -> usize {
        BITBOARD_HEIGHT - self.cols[col].leading_zeros() as usize
    }
    #[inline]
    pub fn stack_height(&self) -> usize {
        let contains_stack = self.cols.iter().fold(0, |acc, &col| acc | col);
        // dbg!(contains_stack);
        // dbg!(self.cols);
        assert_eq!(
            contains_stack.trailing_ones() + contains_stack.leading_zeros(),
            BITBOARD_HEIGHT as u32
        );
        BITBOARD_HEIGHT - contains_stack.leading_zeros() as usize
    }
    pub fn collides(&self, tentative_piece: &FallingPiece) -> bool {
        tentative_piece.coords.iter().any(|&(y, x)| {
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
            MoveLeft => tentative_piece.shift(0, -1),
            MoveRight => tentative_piece.shift(0, 1),
            Drop => tentative_piece.shift(-1, 0),
            SonicDrop => {
                let distance = tentative_piece
                    .coords
                    .iter()
                    .map(|&(y, x)| {
                        if y == 0 {
                            0
                        } else {
                            (!self.cols[x as usize] << (BITBOARD_HEIGHT - y as usize))
                                .leading_ones() as i8
                        }
                    })
                    .min()
                    .unwrap();
                debug_assert!(distance >= 0);
                if distance == 0 {
                    return None;
                } else {
                    tentative_piece.shift(-distance, 0)
                }
            }
            RotateCw | RotateCcw => {
                tentative_piece.rotate(command);

                // go through kicks in kicktable
                for [x_kick, y_kick] in
                    tentative_piece.piece.kicks(command)[tentative_piece.rotation as usize]
                {
                    let mut kicked_piece = tentative_piece;
                    kicked_piece.shift(y_kick, x_kick);
                    if !self.collides(&kicked_piece) {
                        return Some(kicked_piece);
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
        self.try_command(falling_piece, Command::SonicDrop).unwrap_or(*falling_piece)
    }

    /* lock */

    pub fn hard_drop(&self, all_spin: bool, data: BoardData) -> (BitBoard, BoardData) {
        let mut new_board = *self;
        let mut new_data = data;

        let mut cleared_lines = 0;
        let mut rows_to_clear = new_board.cols.iter().fold(u32::MAX, |acc, &col| acc & col);
        rows_to_clear &= u32::MAX << data.simulated_garbage;

        while rows_to_clear != 0 {
            let row = rows_to_clear.trailing_zeros();
            rows_to_clear &= rows_to_clear - 1; // clear lsb
            cleared_lines += 1;
            for col in &mut new_board.cols {
                let mask_below_row = (1 << row) - 1;
                let mask_above_row = !((mask_below_row << 1) + 1);
                let below_row = *col & mask_below_row;
                *col = (*col & mask_above_row) >> 1 | below_row;
            }
            rows_to_clear >>= 1;
        }

        if cleared_lines > 0 {
            new_data.combo += 1;
            new_data.b2b = all_spin || cleared_lines == 4;
            let mut attack = 0;
            use crate::botris::{game_info::*, types::ClearName::*};
            if all_spin {
                attack += match cleared_lines {
                    1 => ASS.attack(),
                    2 => ASD.attack(),
                    3 => AST.attack(),
                    _ => {
                        dbg!(cleared_lines);
                        self.print_board(None);
                        panic!();
                    }
                };
            } else {
                attack += match cleared_lines {
                    1 => Single.attack(),
                    2 => Double.attack(),
                    3 => Triple.attack(),
                    4 => Quad.attack(),
                    _ => panic!(),
                };
            }
            attack += COMBO_TABLE[new_data.combo as usize] as u32;
            if new_board.cols.iter().all(|&x| x == 0) {
                attack += PC.attack();
            }
            if data.b2b && new_data.b2b {
                attack += B2B_ATTACK;
            }
            new_data.cummulative_attack += attack;
            for garb in &mut new_data.incoming {
                if attack <= *garb {
                    *garb -= attack;
                    break;
                } else {
                    attack -= *garb;
                    *garb = 0;
                }
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

#[cfg(test)]
mod test {
    use crate::tetris_core::piece::FallingPiece;

    use super::BitBoard;

    #[test]
    fn test_sonic_drop() {
        let board = BitBoard::from_strs(&[
            "[][]            [][]",
            "[][]        [][][][]",
        ]);
        let dropped = board.force_sonic_drop(&FallingPiece::new(crate::botris::types::Piece::Z));
        board.print_board(Some(dropped.coords));
    }

    #[test]
    fn test_hard_drop() {
        let board = BitBoard::from_strs(&[
            "[][][][][][]    [][]",
            "[][][][][][][][][][]",
            "[][][][][][][][][][]",
            "[][][][][]  [][][][]",
            "[][][][][][][][][][]",
            "[][][][][][][][][][]",
            "[][][]  [][][][][][]",
            "[][]    [][][][][][]",
        ]);
        let (new_board, _data) = board.hard_drop(false, Default::default());
        new_board.print_board(None);
    }
}

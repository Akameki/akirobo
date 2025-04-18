use crate::botris::{game_info::BOARD_HEIGHT, types::Board};

/// index 0 is the bottom of the board
pub type Matrix = [[bool; 10]; BOARD_HEIGHT];

pub const EMPTY_BOARD: Matrix = [[false; 10]; BOARD_HEIGHT];

pub fn display_matrix(board: &Matrix) {
    for (i, row) in board.iter().rev().enumerate() {
        // ignore rows above 21 if empty
        if i >= 21 && row.iter().all(|&cell| !cell) {
            continue;
        }
        for cell in row.iter() {
            print!("{}", if *cell { "[]" } else { "  " });
        }
        println!();
    }
}

// converts board as returned by Botris API to the internal board representation
pub fn to_board(board: &Board) -> Matrix {
    let mut new_board = EMPTY_BOARD;
    for (i, row) in board.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            new_board[i][j] = cell.is_some();
        }
    }
    new_board
}

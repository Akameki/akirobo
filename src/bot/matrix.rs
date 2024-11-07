//! Functions for the game matrix, represented as a [[false; 10]; BOARD_HEIGHT]

use crate::botris::game_info::BOARD_HEIGHT;
use crate::botris::types::{Board, Block, Piece};

pub const EMPTY_BOARD: [[bool; 10]; BOARD_HEIGHT] = [[false; 10]; BOARD_HEIGHT];

pub fn display_board(board: &[[bool; 10]; BOARD_HEIGHT]) {
    println!("====================");
    for row in board.iter().rev() {
        for cell in row.iter() {
            print!("{}", if *cell { "[]" } else { "  " });
        }
        println!();
    }
    println!("====================");
}

// converts board as returned by Botris API to the internal board representation
pub fn to_board(board: &Board) -> [[bool; 10]; BOARD_HEIGHT] {
    let mut new_board = EMPTY_BOARD;
    for (i, row) in board.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            new_board[i][j] = cell.is_some();
        }
    }
    new_board
}
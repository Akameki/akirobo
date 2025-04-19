use super::piece::FallingPiece;
use crate::botris::{
    self,
    game_info::BOARD_HEIGHT,
    types::Command,
};

/// index 0 is the bottom of the board
pub type Board = [[bool; 10]; BOARD_HEIGHT];

pub const EMPTY_BOARD: Board = [[false; 10]; BOARD_HEIGHT];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BoardData {
    pub b2b: bool,
    pub incoming: [u32; 8],
    pub combo: u32,
    pub cummulative_attack: u32,
    pub simulated_garbage: u32,
}

pub fn print_board(board: &Board) {
    let first_empty_row = board.iter().position(|row| row.iter().all(|&cell| !cell));
    let rows_to_print = std::cmp::min(first_empty_row.unwrap_or(BOARD_HEIGHT) + 5, BOARD_HEIGHT);

    println!(">~~~~~~~~~~~~~~~~~~~~<");
    for row in board.iter().take(rows_to_print).rev() {
        print!("\"");
        for cell in row.iter() {
            print!("{}", if *cell { "[]" } else { "  " });
        }
        println!("\"");
    }
    println!(">~~~~~~~~~~~~~~~~~~~~<");
}

// converts board as returned by Botris API to the internal board representation
pub fn to_board(board: &botris::types::Board) -> Board {
    let mut new_board = EMPTY_BOARD;
    for (i, row) in board.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            new_board[i][j] = cell.is_some();
        }
    }
    new_board
}

pub fn strs_to_board(strs: &[&str]) -> Board {
    debug_assert_eq!(strs[0].len(), 20);
    let mut board = EMPTY_BOARD;
    for (y, row) in strs.iter().rev().enumerate() {
        for (x, cell) in row.chars().step_by(2).enumerate() {
            board[y][x] = cell != ' ';
        }
    }
    board
}

pub fn strs_to_board_mini(strs: &[&str]) -> Board {
    debug_assert_eq!(strs[0].len(), 10);
    let mut board = EMPTY_BOARD;
    for (y, row) in strs.iter().rev().enumerate() {
        for (x, cell) in row.chars().enumerate() {
            board[y][x] = cell != ' ';
        }
    }
    board
}

/* functions for manipulating falling piece within board */

pub fn collides(board: &Board, tentative_piece: &FallingPiece) -> bool {
    tentative_piece.absolute().iter().any(|&(y, x)| {
        x < 0 || y < 0 || x >= 10 || y >= BOARD_HEIGHT as i8 || board[y as usize][x as usize]
    })
}

/// returns None if the command is impossible or does nothing.
pub fn try_command(
    board: &Board,
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
            while !collides(board, &tentative_piece) {
                tentative_piece.y -= 1;
            }
            tentative_piece.y += 1;
        }
        RotateCw | RotateCcw => {
            tentative_piece.rotation += if command == RotateCw { 1 } else { 3 };
            tentative_piece.rotation %= 4;

            // go through kicks in kicktable
            for [x_kick, y_kick] in tentative_piece.piece.kicks(command)[tentative_piece.rotation] {
                let rotated = FallingPiece {
                    x: tentative_piece.x + x_kick,
                    y: tentative_piece.y + y_kick,
                    rotation: tentative_piece.rotation,
                    piece: tentative_piece.piece,
                };
                if !collides(board, &rotated) {
                    return Some(rotated);
                }
            }
            return None;
        }
        _ => panic!(),
    }

    if collides(board, &tentative_piece) || tentative_piece == *falling_piece {
        None
    } else {
        Some(tentative_piece)
    }
}

pub fn try_commands(
    board: &Board,
    falling_piece: &FallingPiece,
    commands: &[Command],
) -> Option<FallingPiece> {
    commands
        .iter()
        .try_fold(*falling_piece, |falling, &command| try_command(board, &falling, command))
}

pub fn force_sonic_drop(board: &Board, falling_piece: &FallingPiece) -> FallingPiece {
    let mut tentative_piece = *falling_piece;
    while !collides(board, &tentative_piece) {
        tentative_piece.y -= 1;
    }
    tentative_piece.y += 1;
    tentative_piece
}

/* lock */

pub fn hard_drop(board: &Board, all_spin: bool, data: BoardData) -> (Board, BoardData) {
    let mut new_board = *board;
    let mut new_data = data;

    // clear lines
    let cleared_lines = {
        let mut cleared = 0;
        let mut y = BOARD_HEIGHT - 1;
        while y >= data.simulated_garbage as usize { // TODO: add me back in
            if new_board[y].iter().all(|&x| x) {
                cleared += 1;
                new_board.copy_within((y + 1).., y);
                new_board[BOARD_HEIGHT - 1] = [false; 10];
                // don't decrement y so we re-check the row that was shifted down
            } else {
                if y == 0 {
                    // don't underflow usize
                    break;
                }
                y -= 1;
            }
        }
        cleared
    };

    if cleared_lines > 0 {
        new_data.combo += 1;
        new_data.b2b = all_spin || cleared_lines == 4;
        use crate::botris::types::ClearName::*;
        use crate::botris::game_info::*;
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
        if new_board[0].iter().all(|&x| !x) {
            new_data.cummulative_attack += PC.attack();
        }
        if data.b2b && new_data.b2b {
            new_data.cummulative_attack += B2B_ATTACK;
        }
        new_data.incoming[0] += new_data.incoming[1];
    } else {
        new_data.combo = 0;
        let added_garbage = std::cmp::min(new_data.incoming[0] as usize, BOARD_HEIGHT);
        if added_garbage > 0 {
            new_data.simulated_garbage += added_garbage as u32;
            new_board.copy_within(..(BOARD_HEIGHT - added_garbage), added_garbage);
            for row in &mut new_board[..added_garbage] {
                *row = [true; 10];
            }
        }
        new_data.incoming[0] = 0;
    }
    // incoming[0] properly updated. now shift everything else over by 1
    new_data.incoming.copy_within(2.., 1);

    (new_board, new_data)
}
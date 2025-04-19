use std::collections::{HashMap, HashSet};

use owo_colors::OwoColorize;

use crate::{
    botris::{
        game_info::BOARD_HEIGHT,
        types::{Command, Piece},
    },
    tetris_core::{
        engine::{collides, force_sonic_drop, try_command, try_commands, Board},
        piece::{FallingPiece, PieceCoords},
    },
};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Placement {
    pub piece_location: PieceCoords,
    pub all_spin: bool,
}

impl Placement {
    pub fn new(board: &Board, falling_piece: &FallingPiece) -> Placement {
        let all_spin = [(0, 1), (1, 0), (-1, 0)].iter().all(|(dx, dy)| {
            collides(
                board,
                &FallingPiece {
                    x: falling_piece.x + dx,
                    y: falling_piece.y + dy,
                    ..*falling_piece
                },
            )
        });
        Placement { piece_location: falling_piece.absolute(), all_spin }
    }
}

pub fn move_gen_with_action(board: &Board, piece: Piece) -> HashMap<Placement, Vec<Command>> {
    use Command::*;
    let rotation_sets = [vec![], vec![RotateCw], vec![RotateCcw], vec![RotateCcw, RotateCcw]];

    let initial_falling_piece = FallingPiece::new(piece);
    if collides(board, &initial_falling_piece) {
        return HashMap::new();
    }

    let mut rotated_pieces = HashMap::new();
    for rotation_set in rotation_sets {
        if let Some(rotated_piece) = try_commands(board, &initial_falling_piece, &rotation_set) {
            rotated_pieces.insert(rotated_piece, rotation_set.clone());
        }
    }
    let mut moved_and_soniced = HashMap::new();
    for (piece, mut action) in rotated_pieces {
        for direction in [MoveLeft, MoveRight] {
            let mut moving_piece = piece;
            let mut moving_action = action.clone();
            while let Some(moved_piece) = try_command(board, &moving_piece, direction) {
                moving_piece = moved_piece;
                moving_action.push(direction);
                let mut val = moving_action.clone();
                val.push(SonicDrop);
                moved_and_soniced.insert(force_sonic_drop(board, &moving_piece), val);
            }
        }
        action.push(SonicDrop);
        moved_and_soniced.insert(force_sonic_drop(board, &piece), action);
    }

    let mut generated = HashMap::new();

    for (piece, action) in &moved_and_soniced {
        generated.insert(Placement::new(board, piece), action.clone());
    }
    for spin in [RotateCcw, RotateCw] {
        for (piece, action) in &moved_and_soniced {
            if let Some(spun) = try_command(board, piece, spin) {
                let placement = Placement::new(board, &force_sonic_drop(board, &spun));
                let mut val = action.clone();
                val.push(spin);
                generated.entry(placement).or_insert(val);
            }
        }
    }

    generated
}

pub fn move_gen(board: &Board, piece: Piece) -> HashSet<Placement> {
    use Command::*;
    let rotation_sets = [vec![], vec![RotateCw], vec![RotateCcw], vec![RotateCcw, RotateCcw]];

    let initial_falling_piece = FallingPiece::new(piece);
    if collides(board, &initial_falling_piece) {
        return HashSet::new();
    }

    let mut rotated_pieces = HashSet::new();
    for rotation_set in rotation_sets {
        if let Some(rotated_piece) = try_commands(board, &initial_falling_piece, &rotation_set) {
            rotated_pieces.insert(rotated_piece);
        }
    }
    let mut moved_and_soniced = HashSet::new();
    for piece in rotated_pieces {
        for direction in [MoveLeft, MoveRight] {
            let mut moving_piece = piece;
            while let Some(moved_piece) = try_command(board, &moving_piece, direction) {
                moving_piece = moved_piece;
                moved_and_soniced.insert(force_sonic_drop(board, &moving_piece));
            }
        }
        moved_and_soniced.insert(force_sonic_drop(board, &piece));
    }

    let mut generated = HashSet::new();

    for piece in &moved_and_soniced {
        generated.insert(Placement::new(board, piece));
    }
    for spin in [RotateCcw, RotateCw] {
        for piece in &moved_and_soniced {
            if let Some(spun) = try_command(board, piece, spin) {
                generated.insert(Placement::new(board, &force_sonic_drop(board, &spun)));
            }
        }
    }

    generated
}

pub fn print_placements_from_board<'a, I>(board: &Board, placements: I, chunk_size: usize)
where
    I: IntoIterator<Item = &'a Placement>,
{
    let first_empty_row = board.iter().position(|row| row.iter().all(|&cell| !cell));
    let rows_to_print = std::cmp::min(first_empty_row.unwrap_or(BOARD_HEIGHT) + 5, BOARD_HEIGHT);

    let placements: Vec<&Placement> = placements.into_iter().collect();
    println!("{}", ">~~~~~~~~~~~~~~~~~~~~<".repeat(std::cmp::min(placements.len(), chunk_size)));
    for chunk in placements.chunks(chunk_size) {
        for (y, row) in board.iter().enumerate().take(rows_to_print).rev() {
            for placement in chunk {
                print!("\"");
                for (x, cell) in row.iter().enumerate() {
                    if placement.piece_location.contains(&(y as i8, x as i8)) {
                        if placement.all_spin {
                            print!("{}", "██".fg_rgb::<150, 100, 50>());
                        } else {
                            print!("██");
                        }
                    } else {
                        print!("{}", if *cell { "[]" } else { "  " });
                    }
                }
                print!("\"");
            }
            println!();
        }
        println!("{}", ">~~~~~~~~~~~~~~~~~~~~<".repeat(chunk.len()));
    }
}

pub fn print_placement(board: &Board, placement: &Placement) {
    print_placements_from_board(board, vec![placement], 1);
}

#[cfg(test)]
mod test {
    use super::{move_gen, print_placement};
    use crate::{
        botris::types::Piece,
        movegen::{move_gen_with_action, print_placements_from_board},
        tetris_core::engine::strs_to_board_mini,
    };

    #[test]
    fn test_move_gen() {
        let board = strs_to_board_mini(&["X X XX  X ", "X  XX     "]);
        // let moves = move_gen_with_action(&board, Piece::O);
        // println!("generated {}", moves.len());
        // for (placement, action) in moves {
        //     print_placement(&board, &placement);
        //     println!("{:?}", action);
        // }
        let moves = move_gen_with_action(&board, Piece::L);
        print_placements_from_board(&board, moves.keys(), 5);
        println!("generated {}", moves.len());
    }
}

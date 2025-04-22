use ahash::{AHashMap, AHashSet};

use crate::{
    botris::types::{Command, Piece},
    tetris_core::{
        engine::BitBoard,
        piece::{FallingPiece, PieceCoords},
    },
};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Placement {
    pub piece_location: PieceCoords,
    pub all_spin: bool,
}

impl Placement {
    pub fn new(board: &BitBoard, falling_piece: &FallingPiece) -> Placement {
        let all_spin = [(0, 1), (1, 0), (-1, 0)].iter().all(|&(dx, dy)| {
            let mut nudged = *falling_piece;
            nudged.shift(dy, dx);
            board.collides(&nudged)
        });
        Placement { piece_location: falling_piece.coords, all_spin }
    }
}

pub fn move_gen_with_action(board: &BitBoard, piece: Piece) -> AHashMap<Placement, Vec<Command>> {
    use Command::*;
    let rotation_sets = [vec![], vec![RotateCw], vec![RotateCcw], vec![RotateCcw, RotateCcw]];

    let initial_falling_piece = FallingPiece::new(piece);
    if board.collides(&initial_falling_piece) {
        return AHashMap::new();
    }

    let mut rotated_pieces = AHashMap::new();
    for rotation_set in rotation_sets {
        if let Some(rotated_piece) = board.try_commands(&initial_falling_piece, &rotation_set) {
            rotated_pieces.insert(rotated_piece, rotation_set.clone());
        }
    }
    let mut moved_and_soniced = AHashMap::new();
    for (piece, mut action) in rotated_pieces {
        for direction in [MoveLeft, MoveRight] {
            let mut moving_piece = piece;
            let mut moving_action = action.clone();
            while let Some(moved_piece) = board.try_command(&moving_piece, direction) {
                moving_piece = moved_piece;
                moving_action.push(direction);
                let mut val = moving_action.clone();
                val.push(SonicDrop);
                moved_and_soniced.insert(board.force_sonic_drop(&moving_piece), val);
            }
        }
        action.push(SonicDrop);
        moved_and_soniced.insert(board.force_sonic_drop(&piece), action);
    }

    let mut generated = AHashMap::new();

    for (piece, action) in &moved_and_soniced {
        generated.insert(Placement::new(board, piece), action.clone());
    }
    for spin in [RotateCcw, RotateCw] {
        for (piece, action) in &moved_and_soniced {
            if let Some(spun) = board.try_command(piece, spin) {
                let placement = Placement::new(board, &board.force_sonic_drop(&spun));
                let mut val = action.clone();
                val.push(spin);
                generated.entry(placement).or_insert(val);
            }
        }
    }

    generated
}

pub fn move_gen(board: &BitBoard, piece: Piece) -> AHashSet<Placement> {
    use Command::*;
    let rotation_sets = [vec![], vec![RotateCw], vec![RotateCcw], vec![RotateCcw, RotateCcw]];

    let initial_falling_piece = FallingPiece::new(piece);
    if board.collides(&initial_falling_piece) {
        return AHashSet::new();
    }

    let mut rotated_pieces = AHashSet::new();
    for rotation_set in rotation_sets {
        if let Some(rotated_piece) = board.try_commands(&initial_falling_piece, &rotation_set) {
            rotated_pieces.insert(rotated_piece);
        }
    }
    let mut moved_and_soniced = AHashSet::new();
    for piece in rotated_pieces {
        for direction in [MoveLeft, MoveRight] {
            let mut moving_piece = piece;
            while let Some(moved_piece) = board.try_command(&moving_piece, direction) {
                moving_piece = moved_piece;
                moved_and_soniced.insert(board.force_sonic_drop(&moving_piece));
            }
        }
        moved_and_soniced.insert(board.force_sonic_drop(&piece));
    }

    let mut generated = AHashSet::new();

    for piece in &moved_and_soniced {
        generated.insert(Placement::new(board, piece));
    }
    for spin in [RotateCcw, RotateCw] {
        for piece in &moved_and_soniced {
            if let Some(spun) = board.try_command(piece, spin) {
                generated.insert(Placement::new(board, &board.force_sonic_drop(&spun)));
            }
        }
    }

    generated
}

#[cfg(test)]
mod test {
    use super::{move_gen, move_gen_with_action};
    use crate::{botris::types::Piece, tetris_core::engine::BitBoard};

    #[test]
    fn test_move_gen() {
        let board = BitBoard::from_strs(&[
            "                    ",
            "[]  []  [][]    []  ",
            "[]    []            ",
        ]);
        let moves = move_gen(&board, Piece::L);
        let moves_with_action = move_gen_with_action(&board, Piece::L);
        BitBoard::print_rows(
            &moves
                .iter()
                .map(|placement| (&board, Some(placement.piece_location)))
                .collect::<Vec<_>>(),
            5,
        );
        assert_eq!(moves.len(), moves_with_action.len());
        println!("generated {}", moves_with_action.len());
    }
}

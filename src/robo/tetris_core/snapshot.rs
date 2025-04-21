use std::hash::Hash;

use rand::{seq::SliceRandom, thread_rng};

use super::{engine::*, piece::*};
use crate::botris::types::{GameState, GarbageLine, Piece};

#[derive(Debug, Clone, Eq)]
pub struct GameSnapshot {
    pub matrix: BitBoard,
    pub falling_piece: FallingPiece,
    pub queue: Vec<Piece>,
    pub held: Piece,
    pub can_hold: bool,
    pub combo: u32,
    pub b2b: bool,
    pub incoming_garbage: [u32; 8],
    /// number of lines in the matrix that is treated is unclearable
    pub permanent_garbage: usize,
}

impl PartialEq for GameSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.matrix == other.matrix
            && self.falling_piece == other.falling_piece
            && self.held == other.held
            && self.can_hold == other.can_hold
            && self.combo == other.combo
            && self.b2b == other.b2b
            && self.incoming_garbage == other.incoming_garbage
    }
}
impl Hash for GameSnapshot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.matrix.hash(state);
        self.falling_piece.hash(state);
        self.held.hash(state);
        self.can_hold.hash(state);
        self.combo.hash(state);
        self.b2b.hash(state);
        self.incoming_garbage.hash(state);
    }
}

impl GameSnapshot {
    pub fn from_state(game_state: &GameState) -> Self {
        let mut queue = game_state.queue.clone();
        queue.extend(game_state.bag.clone());
        for _ in 0..5 {
            let mut random_bag =
                [Piece::I, Piece::J, Piece::L, Piece::O, Piece::S, Piece::T, Piece::Z];
            random_bag.shuffle(&mut thread_rng());
            queue.extend(random_bag);
        }

        let mut incoming = [0; 8];
        for GarbageLine { delay } in &game_state.garbage_queued {
            // assume we can play above 2pps
            let delay = delay.as_u64().unwrap() as u32 * 2;
            assert!(delay < 8);
            incoming[delay as usize] += 1;
        }

        GameSnapshot {
            matrix: to_board(&game_state.board),
            queue,
            held: game_state.held.expect("no held piece in Frame"),
            falling_piece: FallingPiece::new(game_state.current.piece),
            can_hold: game_state.can_hold,
            combo: game_state.combo,
            b2b: game_state.b2b,
            incoming_garbage: incoming,
            permanent_garbage: 0,
        }
    }
}

//             Hold => {
//                 if !self.can_hold || self.held == self.falling_piece.piece {
//                     return None;
//                 } else {
//                     return Some(SomeGameState {
//                         held: self.falling_piece.piece,
//                         falling_piece: FallingPiece::new(self.held),
//                         can_hold: false,
//                         confirmed_on_bottom: false,
//                         ..self.clone()
//                     });
//                 }
//             }

//     // print frames side by side
//     pub fn print_frames(frames: &[SomeGameState], max_per_row: usize) {
//         for chunk in frames.chunks(max_per_row) {
//             // print held piece and next 5 in queue
//             for frame in chunk {
//                 print!(
//                     "\"[{:?}]{}      {:?} {:?}{:?}{:?}{:?} \"",
//                     frame.held,
//                     if frame.can_hold { "    " } else { "HELD" },
//                     frame.queue[0],
//                     frame.queue[1],
//                     frame.queue[2],
//                     frame.queue[3],
//                     frame.queue[4]
//                 );
//             }
//             println!();

//             for y in (0..BOARD_HEIGHT).rev() {
//                 if y >= 21 && chunk.iter().all(|frame| frame.matrix[y].iter().all(|&cell| !cell)) {
//                     continue;
//                 }
//                 for frame in chunk {
//                     print!("\"");
//                     for x in 0..10 {
//                         if frame.falling_piece.absolute().contains(&(y as i8, x as i8)) {
//                             print!("██");
//                         } else {
//                             print!("{}", if frame.matrix[y][x] { "[]" } else { "  " });
//                         }
//                     }
//                     print!("\"");
//                 }
//                 println!();
//             }
//             println!();
//         }
//     }

impl Default for GameSnapshot {
    fn default() -> Self {
        GameSnapshot {
            matrix: EMPTY_BOARD,
            falling_piece: FallingPiece::new(Piece::I),
            queue: vec![],
            held: Piece::O,
            can_hold: true,
            combo: 0,
            b2b: false,
            incoming_garbage: [0; 8],
            permanent_garbage: 0,
        }
    }
}

// impl fmt::Display for SomeGameState {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         writeln!(f, "====================")?;
//         let matrix = self.matrix;
//         for y in (0..matrix.len()).rev() {
//             // ignore rows above 21 if empty
//             if y >= 21 && matrix[y].iter().all(|&cell| !cell) {
//                 continue;
//             }
//             for x in 0..10 {
//                 if self.falling_piece.absolute().contains(&(y as i8, x as i8)) {
//                     write!(f, "██")?;
//                 } else {
//                     write!(f, "{}", if matrix[y][x] { "[]" } else { "  " })?;
//                 }
//             }
//             writeln!(f)?;
//         }
//         write!(f, "====================")
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::SomeGameState;
//     use crate::{
//         botris::types::Piece,
//         tetris_core::piece::FallingPiece,
//     };

//     #[test]
//     fn play_moves() {
//         let mut frame = SomeGameState {
//             falling_piece: FallingPiece::new(Piece::O),
//             queue: vec![Piece::O, Piece::O, Piece::O, Piece::O, Piece::O],
//             held: Piece::O,
//             incoming: vec![[3, 0], [2, 2], [1, 3]],
//             ..Default::default()
//         };
//         // let moves =
//         //     vec![RotateCcw, RotateCcw, RotateCcw, RotateCcw, MoveRight, MoveRight, MoveRight];
//         // println!("{}", frame);
//         for _ in 0..4 {
//             // println!("Running command: {:?}", command);
//             frame = frame.force_sonic_drop();
//             println!("{}", frame);
//             frame = frame.lock_piece().unwrap();
//             println!("{}", frame)
//         }
//     }
// }

//! Represents the current frame of the game
//! This includes the playfield, current piece, queue/hold
//! And also notably the attack that the player would send since the game state from a request
//!
//! Also includes methods to run commands and display the frame

use core::fmt;
use std::{cmp::min, hash::Hash};

use rand::{seq::SliceRandom, thread_rng};

use super::{matrix::*, piece::*};
use crate::botris::{
    game_info::{B2B_ATTACK, BOARD_HEIGHT, COMBO_TABLE},
    types::{
        ClearName::*,
        Command::{self, *},
        GameState, GarbageLine, Piece,
    },
};

#[derive(Debug, Clone, Eq)]
pub struct Frame {
    pub matrix: Matrix,
    pub falling_piece: FallingPiece,
    pub queue: Vec<Piece>,
    pub held: Piece,
    pub can_hold: bool,
    pub combo: u32,
    pub b2b: bool,
    pub stored_attack: u32,
    pub incoming: Vec<[u16; 2]>,
    /// number of lines in the matrix that is treated is unclearable
    pub simulated_garbage: usize,

    pub confirmed_on_bottom: bool,
    // pub dropped: bool, // only allow evaluating when the piece has been dropped
    pub depth: usize,
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth
            && self.matrix == other.matrix
            && self.falling_piece == other.falling_piece
            && self.held == other.held
            && self.can_hold == other.can_hold
            && self.combo == other.combo
            && self.b2b == other.b2b
            && self.incoming == other.incoming
            && self.confirmed_on_bottom == other.confirmed_on_bottom
            && self.stored_attack == other.stored_attack
    }
}
impl Hash for Frame {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.depth.hash(state);
        self.matrix.hash(state);
        self.falling_piece.hash(state);
        self.held.hash(state);
        self.can_hold.hash(state);
        self.combo.hash(state);
        self.b2b.hash(state);
        self.incoming.hash(state);
        self.stored_attack.hash(state);
        self.confirmed_on_bottom.hash(state);
    }
}

impl Frame {
    pub fn from_state(game_state: &GameState) -> Self {
        let mut queue = game_state.queue.clone();
        queue.extend(game_state.bag.clone());
        for _ in 0..5 {
            let mut random_bag =
                [Piece::I, Piece::J, Piece::L, Piece::O, Piece::S, Piece::T, Piece::Z];
            random_bag.shuffle(&mut thread_rng());
            queue.extend(random_bag);
        }

        let mut incoming: Vec<[u16; 2]> = Vec::new();
        let mut current = [0, 0];
        for GarbageLine { delay } in &game_state.garbage_queued {
            // assume we can play above 2pps
            let delay = delay.as_u64().unwrap() as u16 * 2;
            if delay != current[1] {
                if current != [0, 0] {
                    incoming.push(current);
                }
                current = [1, delay];
            } else {
                current[0] += 1;
            }
        }
        if current != [0, 0] {
            incoming.push(current);
        }

        Frame {
            matrix: to_board(&game_state.board),
            // bag: game_state.bag.clone(),
            queue,
            held: game_state.held.expect("no held piece in Frame"),
            falling_piece: FallingPiece::new(game_state.current.piece),
            can_hold: game_state.can_hold,
            combo: game_state.combo,
            b2b: game_state.b2b,
            stored_attack: 0,
            incoming,
            simulated_garbage: 0,
            // dropped: false,
            confirmed_on_bottom: false,
            depth: 0,
            // first_action: None,
        }
    }

    fn collides_with_board(board: &Matrix, tentative_piece: &FallingPiece) -> bool {
        tentative_piece.absolute().iter().any(|&(y, x)| {
            x < 0 || y < 0 || x >= 10 || y >= BOARD_HEIGHT as i8 || board[y as usize][x as usize]
        })
    }

    /// returns None if the command is impossible or does nothing.
    pub fn try_command(&self, command: Command) -> Option<Self> {
        let mut confirmed_on_bottom = false;
        let mut tentative_piece = self.falling_piece.clone();
        match command {
            MoveLeft => tentative_piece.x -= 1,
            MoveRight => tentative_piece.x += 1,
            Drop => tentative_piece.y -= 1,
            RotateCw | RotateCcw => {
                tentative_piece.rotation += if command == RotateCw { 1 } else { 3 };
                tentative_piece.rotation %= 4;

                // go through kicks in kicktable
                for [x_kick, y_kick] in
                    tentative_piece.piece.kicks(command)[tentative_piece.rotation]
                {
                    tentative_piece.x += x_kick;
                    tentative_piece.y += y_kick;
                    if !Self::collides_with_board(&self.matrix, &tentative_piece) {
                        break;
                    }
                    tentative_piece.x -= x_kick;
                    tentative_piece.y -= y_kick;
                }
                // if no kicks work (rotation fails), collision is checked before return!
            }
            SonicDrop => {
                while !Self::collides_with_board(&self.matrix, &tentative_piece) {
                    tentative_piece.y -= 1;
                }
                tentative_piece.y += 1;
                confirmed_on_bottom = true;
            }
            Hold => {
                if !self.can_hold || self.held == self.falling_piece.piece {
                    return None;
                } else {
                    return Some(Frame {
                        held: self.falling_piece.piece,
                        falling_piece: FallingPiece::new(self.held),
                        can_hold: false,
                        confirmed_on_bottom: false,
                        ..self.clone()
                    });
                }
            }
            _ => panic!(),
        }

        if Self::collides_with_board(&self.matrix, &tentative_piece)
            || tentative_piece == self.falling_piece
        {
            None
        } else {
            Some(Frame { falling_piece: tentative_piece, confirmed_on_bottom, ..self.clone() })
        }
    }

    pub fn try_commands(&self, commands: &[Command]) -> Option<Self> {
        commands.iter().try_fold(self.clone(), |frame, &command| frame.try_command(command))
    }

    pub fn force_sonic_drop(&self) -> Frame {
        let mut tentative_piece = self.falling_piece.clone();
        while !Self::collides_with_board(&self.matrix, &tentative_piece) {
            tentative_piece.y -= 1;
        }
        tentative_piece.y += 1;
        Frame { falling_piece: tentative_piece, confirmed_on_bottom: true, ..self.clone() }
    }

    /// clearing lines and calculating attack, advancing to the next piece
    pub fn lock_piece(&self) -> Option<Frame> {
        if Self::collides_with_board(&self.matrix, &self.falling_piece) {
            return None;
        }

        let mut new_matrix = self.matrix;
        // let mut tentative_piece = self.falling_piece.clone();
        // while !Self::collides_with_board(&self.matrix, &tentative_piece) {
        //     tentative_piece.y -= 1;
        // }
        // tentative_piece.y += 1;
        assert!(self.confirmed_on_bottom);

        // check for all spin (immobilility) (no need to check down direction)
        let all_spin = [(0, 1), (1, 0), (-1, 0)].iter().all(|(dx, dy)| {
            Self::collides_with_board(
                &self.matrix,
                &FallingPiece {
                    x: self.falling_piece.x + dx,
                    y: self.falling_piece.y + dy,
                    ..self.falling_piece
                },
            )
        });

        // lock piece into matrix
        for (y, x) in self.falling_piece.absolute() {
            new_matrix[y as usize][x as usize] = true;
        }

        // clear lines
        let cleared = {
            let mut cleared = 0;
            let mut y = BOARD_HEIGHT - 1;
            while y >= self.simulated_garbage {
                if new_matrix[y].iter().all(|&x| x) {
                    cleared += 1;
                    new_matrix.copy_within((y + 1).., y);
                    new_matrix[BOARD_HEIGHT - 1] = [false; 10];
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

        // calculate attack
        let mut this_attack = 0;
        let mut new_incoming = vec![];
        let mut additional_garbage: usize = 0;

        if cleared > 0 {
            if all_spin {
                this_attack += match cleared {
                    1 => ASS.attack(),
                    2 => ASD.attack(),
                    3 => AST.attack(),
                    _ => panic!(),
                };
            } else {
                this_attack += match cleared {
                    1 => Single.attack(),
                    2 => Double.attack(),
                    3 => Triple.attack(),
                    4 => Quad.attack(),
                    _ => panic!(),
                };
            }
            this_attack += COMBO_TABLE[self.combo as usize] as u32;
            if new_matrix[0].iter().all(|&x| !x) {
                this_attack += PC.attack();
            }
            if self.b2b && (cleared == 4 || all_spin) {
                this_attack += B2B_ATTACK;
            }
            for [lines, delay] in &self.incoming {
                new_incoming.push([*lines, if *delay == 0 { 0 } else { *delay - 1 }])
            }
        } else {
            for [lines, delay] in &self.incoming {
                if *delay == 0 {
                    additional_garbage += *lines as usize;
                } else {
                    new_incoming.push([*lines, *delay - 1]);
                }
            }
            if additional_garbage > 0 {
                additional_garbage = min(additional_garbage, BOARD_HEIGHT);
                new_matrix.copy_within(..(BOARD_HEIGHT - additional_garbage), additional_garbage);
                for row in &mut new_matrix[..additional_garbage] {
                    *row = [true; 10];
                }
            }
        }

        if Self::collides_with_board(&new_matrix, &FallingPiece::new(self.queue[0])) {
            return None;
        }

        Some(Frame {
            matrix: new_matrix,
            queue: self.queue[1..].to_vec(),
            held: self.held,
            falling_piece: FallingPiece::new(self.queue[0]),
            can_hold: true,
            combo: if cleared > 0 { self.combo + 1 } else { 0 },
            b2b: (self.b2b && cleared == 0 ) || cleared == 4 || all_spin,
            stored_attack: self.stored_attack + this_attack,
            incoming: new_incoming,
            simulated_garbage: self.simulated_garbage + additional_garbage,
            depth: self.depth + 1,
            confirmed_on_bottom: false,
        })
    }

    // print frames side by side
    pub fn print_frames(frames: &[Frame], max_per_row: usize) {
        for chunk in frames.chunks(max_per_row) {
            // print held piece and next 5 in queue
            for frame in chunk {
                print!(
                    "\"[{:?}]{}      {:?} {:?}{:?}{:?}{:?} \"",
                    frame.held,
                    if frame.can_hold { "    " } else { "HELD" },
                    frame.queue[0],
                    frame.queue[1],
                    frame.queue[2],
                    frame.queue[3],
                    frame.queue[4]
                );
            }
            println!();

            for y in (0..BOARD_HEIGHT).rev() {
                if y >= 21 && chunk.iter().all(|frame| frame.matrix[y].iter().all(|&cell| !cell)) {
                    continue;
                }
                for frame in chunk {
                    print!("\"");
                    for x in 0..10 {
                        if frame.falling_piece.absolute().contains(&(y as i8, x as i8)) {
                            print!("██");
                        } else {
                            print!("{}", if frame.matrix[y][x] { "[]" } else { "  " });
                        }
                    }
                    print!("\"");
                }
                println!();
            }
            println!();
        }
    }

    // create Frame from Strings (rows) of "  " and non-space "??" chars.
    // append empty rows above to meet board height.
    pub fn from_strings(rows: &[&str]) -> Frame {
        assert!(rows.len() <= BOARD_HEIGHT);
        let mut matrix = EMPTY_BOARD;
        for (y, row) in rows.iter().rev().enumerate() {
            for (x, cell) in row.chars().step_by(2).enumerate() {
                matrix[y][x] = cell != ' ';
            }
        }
        Frame { matrix, ..Default::default() }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Frame {
            matrix: EMPTY_BOARD,
            falling_piece: FallingPiece::new(Piece::I),
            queue: vec![],
            held: Piece::O,
            can_hold: true,
            combo: 0,
            b2b: false,
            stored_attack: 0,
            depth: 0,
            confirmed_on_bottom: false,
            incoming: vec![],
            simulated_garbage: 0,
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "====================")?;
        let matrix = self.matrix;
        for y in (0..matrix.len()).rev() {
            // ignore rows above 21 if empty
            if y >= 21 && matrix[y].iter().all(|&cell| !cell) {
                continue;
            }
            for x in 0..10 {
                if self.falling_piece.absolute().contains(&(y as i8, x as i8)) {
                    write!(f, "██")?;
                } else {
                    write!(f, "{}", if matrix[y][x] { "[]" } else { "  " })?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "====================")
    }
}

#[cfg(test)]
mod test {
    use super::Frame;
    use crate::{
        botris::types::{Command::*, Piece},
        game::piece::FallingPiece,
    };

    #[test]
    fn play_moves() {
        let mut frame = Frame {
            falling_piece: FallingPiece::new(Piece::O),
            queue: vec![Piece::O, Piece::O, Piece::O, Piece::O, Piece::O],
            held: Piece::O,
            incoming: vec![[3, 0], [2, 2], [1, 3]],
            ..Default::default()
        };
        // let moves =
        //     vec![RotateCcw, RotateCcw, RotateCcw, RotateCcw, MoveRight, MoveRight, MoveRight];
        // println!("{}", frame);
        for _ in 0..4 {
            // println!("Running command: {:?}", command);
            frame = frame.force_sonic_drop();
            println!("{}", frame);
            frame = frame.lock_piece().unwrap();
            println!("{}", frame)
        }
    }
}

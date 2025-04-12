//! Represents the current frame of the game
//! This includes the playfield, current piece, queue/hold
//! And also notably the attack that the player would send since the game state from a request
//!
//! Also includes methods to run commands and display the frame

use core::fmt;

// use owo_colors::OwoColorize;

use rand::{seq::SliceRandom, thread_rng};

use super::{matrix::*, piece::*};
use crate::botris::{
    game_info::{BOARD_HEIGHT, COMBO_TABLE},
    types::{ClearName::*, Command, Command::*, GameState, Piece},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Frame {
    pub matrix: Matrix,
    pub falling_piece: FallingPiece,
    pub queue: Vec<Piece>,
    pub held: Piece,
    pub can_hold: bool,
    pub combo: u32,
    pub b2b: bool,
    pub future_attack: u32,

    // pub dropped: bool, // only allow evaluating when the piece has been dropped
    pub depth: usize,
}

impl Frame {
    pub fn from_state(game_state: &GameState) -> Self {
        let mut queue = game_state.queue.clone();
        queue.extend(game_state.bag.clone());
        for _ in 0..5 {
            let mut random_bag= [Piece::I, Piece::J, Piece::L, Piece::O, Piece::S, Piece::T, Piece::Z];
            random_bag.shuffle(&mut thread_rng());
            queue.extend(random_bag);
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
            future_attack: 0,
            // dropped: false,
            depth: 0,
            // first_action: None,
        }
    }

    fn collides(&self, tentative_piece: &FallingPiece) -> bool {
        for (y, x) in tentative_piece.absolute() {
            if x < 0
                || y < 0
                || x >= 10
                || y >= BOARD_HEIGHT as i8
                || self.matrix[y as usize][x as usize]
            {
                return true;
            }
        }
        false
    }

    fn try_rotate(&self, command: Command) -> Option<Self> {
        let mut tentative_piece = self.falling_piece.clone();
        match command {
            RotateCw => tentative_piece.rotation = (tentative_piece.rotation + 1) % 4,
            RotateCcw => tentative_piece.rotation = (tentative_piece.rotation + 3) % 4,
            _ => panic!(),
        }

        // go through kicks in kicktable
        for [dx, dy] in tentative_piece.piece.kicks(command)[tentative_piece.rotation] {
            tentative_piece.x += dx;
            tentative_piece.y += dy;
            if !self.collides(&tentative_piece) {
                return Some(Frame { falling_piece: tentative_piece, ..self.clone() });
            }
            tentative_piece.x -= dx;
            tentative_piece.y -= dy;
        }
        // no valid rotation
        None
    }

    pub fn run_command(&self, command: Command) -> Self {
        self.try_command(command).unwrap_or(self.clone())
    }

    // will not place a piece
    // returns None if piece does not move
    pub fn try_command(&self, command: Command) -> Option<Self> {
        let mut tentative_piece = self.falling_piece.clone();
        match command {
            MoveLeft => tentative_piece.x -= 1,
            MoveRight => tentative_piece.x += 1,
            Drop => tentative_piece.y -= 1,
            RotateCw => return self.try_rotate(command),
            RotateCcw => return self.try_rotate(command),
            SonicDrop => {
                while !self.collides(&tentative_piece) {
                    tentative_piece.y -= 1;
                }
                tentative_piece.y += 1;
            }
            Hold => {
                // Assume always start game with held piece // todo
                if !self.can_hold || self.held == self.falling_piece.piece {
                    return None;
                }
                let held_piece = self.held;
                tentative_piece = FallingPiece::new(held_piece);
                return Some(Frame {
                    held: self.falling_piece.piece,
                    falling_piece: tentative_piece,
                    can_hold: false,
                    ..self.clone()
                });
            }
            _ => panic!(),
        }

        if self.collides(&tentative_piece) || tentative_piece == self.falling_piece {
            None
        } else {
            Some(Frame { falling_piece: tentative_piece, ..self.clone() })
        }
    }

    pub fn try_commands(&self, commands: &[Command]) -> Option<Self> {
        let mut frame = self.clone();
        for command in commands {
            if let Some(new_frame) = frame.try_command(*command) {
                frame = new_frame;
            } else {
                return None;
            }
        }
        Some(frame)
    }

    pub fn force_sonic_drop(&self) -> Frame {
        let mut tentative_piece = self.falling_piece.clone();
        while !self.collides(&tentative_piece) {
            tentative_piece.y -= 1;
        }
        tentative_piece.y += 1;
        Frame { falling_piece: tentative_piece, ..self.clone() }
    }

    // hard drops the piece, changing the matrix and moving to the next piece
    pub fn hard_drop(&self) -> Frame {
        let mut new_matrix = self.matrix;
        let mut tentative_piece = self.falling_piece.clone();
        while !self.collides(&tentative_piece) {
            tentative_piece.y -= 1;
        }
        tentative_piece.y += 1;

        // check for all spin (immobilility)
        let mut all_spin = true;
        for (dy, dx) in &[(0, 0), (0, 1), (1, 0), (1, 1)] {
            let nudged_piece = FallingPiece {
                y: tentative_piece.y + dy,
                x: tentative_piece.x + dx,
                ..tentative_piece
            };
            if self.collides(&nudged_piece) {
                all_spin = false;
                break;
            }
        }

        for (y, x) in tentative_piece.absolute() {
            new_matrix[y as usize][x as usize] = true;
        }

        // clear lines
        let mut cleared = 0;
        for y in 0..BOARD_HEIGHT {
            if new_matrix[y].iter().all(|&x| x) {
                cleared += 1;
                // shift rows down
                for i in y..BOARD_HEIGHT - 1 {
                    new_matrix[i] = new_matrix[i + 1];
                }
                new_matrix[BOARD_HEIGHT - 1] = [false; 10];
            }
        }

        // calculate attack
        let mut future_attack = self.future_attack;

        if all_spin {
            future_attack += match cleared {
                0 => 0,
                1 => 4,
                2 => 4,
                3 => 6,
                _ => panic!(),
            };
        } else {
            future_attack += match cleared {
                0 => 0,
                1 => Single.attack(),
                2 => Double.attack(),
                3 => Triple.attack(),
                4 => Quad.attack(),
                _ => panic!(),
            };
        }

        // add combo
        future_attack += COMBO_TABLE[self.combo as usize] as u32;

        future_attack += self.b2b as u32;
        // lazy pc check
        if new_matrix[0..3].iter().all(|row| row.iter().all(|&x| !x)) {
            future_attack += 10;
        }

        Frame {
            matrix: new_matrix,
            queue: self.queue[1..].to_vec(),
            held: self.held,
            falling_piece: FallingPiece::new(self.queue[0]),
            can_hold: true,
            combo: if cleared > 0 { self.combo + 1 } else { 0 },
            b2b: cleared == 4 || (cleared == 0 && self.b2b) || (cleared > 0 && all_spin),
            future_attack,
            // dropped: true,
            depth: self.depth + 1,
            // first_action: self.first_action.clone(),
        }
    }

    // Print the board in a pretty fashion:
    // At most 3 lines of empty rows
    // Print the falling piece in color so it's visible
    // etc.
    // pub fn pretty_print(&self) {
    //     println!("--------------------");
    //     let mut pretty_matrix_string: [[char; 20]; BOARD_HEIGHT] = [[' '; 20]; BOARD_HEIGHT];
    //     let mut stack_height = 0;
    //     for (y, x) in self.falling_piece.absolute() {
    //         pretty_matrix_string[y as usize][x as usize * 2] = '(';
    //         pretty_matrix_string[y as usize][x as usize * 2 + 1] = ')';
    //     }
    //     for y in (0..BOARD_HEIGHT).rev() {
    //         // ignore rows above 21 if empty
    //         if y >= 21 && self.matrix[y].iter().all(|&cell| !cell) {
    //             continue;
    //         }
    //         for x in 0..10 {
    //             if self.matrix[y][x] {
    //                 stack_height = std::cmp::max(stack_height, y);
    //                 pretty_matrix_string[y][x * 2] = '[';
    //                 pretty_matrix_string[y][x * 2 + 1] = ']';
    //             }
    //         }
    //     }

    //     // print lines with falling piece, and skip until within 2 lines of stack
    //     for y in (0..BOARD_HEIGHT).rev() {
    //         // ignore rows above 21 if empty
    //         if y >= 21 && self.matrix[y].iter().all(|&cell| !cell) {
    //             continue;
    //         }
    //         // print the line with the falling piece
    //         for x in 0..20 {
    //             print!("{}", pretty_matrix_string[y][x].to_string().green());
    //         }
    //         println!();
    //     }
    // }

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
    use crate::{botris::types::{Command::*, GameState, Piece}, game::{matrix::{Matrix, EMPTY_BOARD}, piece::FallingPiece}};

    use super::Frame;

    #[test]
    fn play_moves() {
        let mut frame = Frame {
            matrix: EMPTY_BOARD,
            falling_piece: FallingPiece::new(Piece::O),
            queue: vec![Piece::O, Piece::O, Piece::O],
            held: Piece::O,
            can_hold: true,
            combo: 0,
            b2b: false,
            future_attack: 0,
            depth: 0,
        };
        let moves = vec![RotateCcw, RotateCcw, RotateCcw, RotateCcw, MoveRight, MoveRight, MoveRight];
        println!("{}", frame);
        for command in moves {
            println!("Running command: {:?}", command);
            frame = frame.run_command(command);
            println!("{}", frame);
        }
    }
}
//! Represents the current frame of the game
//! This includes the playfield, current piece, queue/hold
//! And also notably the attack that the player would send since the game state from a request
//! 
//! Also includes methods to run commands and display the frame

use crate::botris::game_info::BOARD_HEIGHT;
use crate::botris::types::ClearName::*;
use crate::botris::types::Command;
use crate::botris::types::Command::*;
use crate::botris::types::GameState;
use crate::botris::types::Piece;

use super::piece::*;
use super::matrix::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Frame {
    pub matrix: Matrix,
    // pub bag: Vec<Piece>,
    pub queue: Vec<Piece>,
    pub held: Option<Piece>,
    pub current: CurrentPiece,
    pub can_hold: bool,
    pub combo: u32,
    pub b2b: bool,
    pub future_attack: u32,

    pub dropped: bool, // only allow evaluating when the piece has been dropped
}

impl Frame {
    pub fn from_state(game_state: &GameState) -> Self {
        Frame {
            matrix: to_board(&game_state.board),
            // bag: game_state.bag.clone(),
            queue: game_state.queue.clone(),
            held: game_state.held,
            current: CurrentPiece::new(game_state.current.piece),
            can_hold: game_state.can_hold,
            combo: game_state.combo,
            b2b: game_state.b2b,
            future_attack: 0,
            dropped: false,
        }
    }

    fn collides(&self, tentative_piece: &CurrentPiece) -> bool {
        for (y, x) in tentative_piece.absolute() {
            if x < 0 || y < 0 || x >= 10 || y >= BOARD_HEIGHT as i8 || self.matrix[y as usize][x as usize] {
                return true;
            }
        }
        false
    }

    // will not place a piece
    // returns None if piece does not move
    pub fn run_command(&self, command: &Command) -> Option<Self> {
        let mut tentative_piece = self.current.clone();
        match command {
            MoveLeft => tentative_piece.x -= 1,
            MoveRight => tentative_piece.x += 1,
            Drop => tentative_piece.y -= 1,
            RotateCw => tentative_piece.rotation = (tentative_piece.rotation + 1) % 4,
            RotateCcw => tentative_piece.rotation = (tentative_piece.rotation + 3) % 4,
            SonicDrop => {
                while !self.collides(&tentative_piece) {
                    tentative_piece.y -= 1;
                }
                tentative_piece.y += 1;
            },
            _ => panic!()
        }

        if self.collides(&tentative_piece) || tentative_piece == self.current {
            if tentative_piece == self.current {
                println!("tentative piece is the same as current piece");
            }
            None
        } else {
            Some(Frame{ current: tentative_piece, ..self.clone() })
        }

    }
    
    pub fn run_commands(&self, commands: &Vec<Command>) -> Option<Self> {
        let mut frame = self.clone();
        for command in commands {
            if let Some(new_frame) = frame.run_command(command) {
                frame = new_frame;
            } else {
                return None;
            }
        }
        Some(frame)
    }

    // hard drops the piece, changing the matrix and moving to the next piece
    pub fn hard_drop(&self) -> Frame {

        let mut new_matrix = self.matrix;
        let mut tentative_piece = self.current.clone();
        while !self.collides(&tentative_piece) {
            tentative_piece.y -= 1;
        }
        tentative_piece.y += 1;
        
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
        // calculate attack (need to check for spins later)
        let mut future_attack = self.future_attack + match cleared {
            0 => 0,
            1 => Single.attack(),
            2 => Double.attack(),
            3 => Triple.attack(),
            4 => Quad.attack(),
            _ => panic!(),
        };
        future_attack += self.b2b as u32;

        Frame {
            matrix: new_matrix,
            queue: self.queue.clone(),
            held: self.held,
            current: tentative_piece,
            can_hold: true,
            combo: 0, // todo
            b2b: cleared == 4,  // todo
            future_attack,
            dropped: true,
        }
    }

    // move onto the next piece (only when dropped)
    pub fn advance(&self) -> Frame {
        if !self.dropped {
            panic!("Cannot advance without dropping the piece");
        }
        let mut new_frame = self.clone();
        new_frame.dropped = false;
        new_frame.current = CurrentPiece::new(new_frame.queue[0]);
        new_frame.queue = new_frame.queue[1..].to_vec();
        new_frame
    }


    pub fn display(&self) {
        println!("====================");
        let mut matrix = self.matrix;

        // for (y, x) in self.current.absolute() {
        //     matrix[y as usize][x as usize] = false; // Clear the current piece position in the matrix
        // }

        for y in (0..matrix.len()).rev() {
            // ignore rows above 21 if empty
            if y >= 21 && matrix[y].iter().all(|&cell| !cell) {
                continue;
            }
            for x in 0..10 {
                if self.current.absolute().contains(&(y as i8, x as i8)) {
                    print!("()");
                } else {
                    print!("{}", if matrix[y][x] { "[]" } else { "  " });
                }
            }
            println!();
        }

        println!("====================");
    }
}
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

#[derive(Debug, Clone)]
pub struct Frame {
    pub matrix: Matrix,
    // pub bag: 
    // combined_matrix: Matrix,
    pub held: Option<Piece>,
    pub current_piece: CurrentPiece,
    pub future_attack: u32,
    pub combo: u32,
    pub b2b: bool,
}

impl Frame {
    // pub fn empty() -> Self {
    //     Engine {
    //         matrix: EMPTY_BOARD,
    //         current_piece: None,
    //     }
    // }

    pub fn new(game_state: &GameState) -> Self {
        Frame {
            matrix: to_board(&game_state.board),
            held: game_state.held,
            current_piece: CurrentPiece::new(game_state.current.piece),
            future_attack: 0,
            combo: game_state.combo,
            b2b: game_state.b2b,
        }
    }

    fn collides(&mut self, tentative_piece: CurrentPiece) -> bool {
        for (y, x) in tentative_piece.absolute() {
            if x < 0 || y < 0 || x >= 10 || y >= BOARD_HEIGHT as i8 || self.matrix[y as usize][x as usize] {
                return true;
            }
        }
        self.current_piece = tentative_piece;
        false
    }

    fn try_step(&mut self, command: Command) -> bool {
        let mut tentative_piece = self.current_piece.clone();
        match command {
            MoveLeft => tentative_piece.x -= 1,
            MoveRight => tentative_piece.x += 1,
            Drop => tentative_piece.y -= 1,
            _ => panic!(),
        }
        !self.collides(tentative_piece)
    }
    fn try_rotate(&mut self, direction: usize) -> bool {
        let mut tentative_piece = self.current_piece.clone();
        tentative_piece.rotation = (tentative_piece.rotation + direction) % 4;
        !self.collides(tentative_piece)
    }
    
    // returns true if the piece moved.
    pub fn try_command(&mut self, command: Command) -> bool {
        match command {
            MoveLeft  => self.try_step(MoveLeft),
            MoveRight => self.try_step(MoveRight),
            RotateCw  => self.try_rotate(1),
            RotateCcw => self.try_rotate(3),
            Drop      => self.try_step(Drop),
            SonicDrop | HardDrop => {
                let ret = self.try_step(Drop);
                while self.try_step(Drop) {};
                ret
            },
            // Command::Hold => {
            //     if let Some(held) = self.held {
            //         self.held = Some(self.current_piece.piece);
            //         self.current_piece = CurrentPiece::new(held);
            //     } else {
            //         self.held = Some(self.current_piece.piece);
            //         self.current_piece = CurrentPiece::new(self.current_piece.piece);
            //     }
            // },
            _ => panic!(),
        }
    }
    
    pub fn run_commands(&mut self, commands: Vec<Command>) -> &mut Self {
        for command in commands {
            self.try_command(command);
        }
        self
    }

    // hard drops the piece, changing the matrix and moving to the next piece
    pub fn place(&mut self) -> &mut Self {
        let mut matrix = self.matrix;
        self.try_command(Command::HardDrop);

        for (y, x) in self.current_piece.absolute() {
            matrix[y as usize][x as usize] = true;
        }

        // clear lines
        let mut cleared = 0;
        for y in 0..BOARD_HEIGHT {
            if matrix[y].iter().all(|&x| x) {
                cleared += 1;
                // shift rows down
                for i in (y..BOARD_HEIGHT - 1) {
                    matrix[i] = matrix[i + 1];
                }
                matrix[BOARD_HEIGHT - 1] = [false; 10];
            }
        }
        // calculate attack
        self.future_attack += match cleared {
            0 => 0,
            1 => Single.attack(),
            2 => Double.attack(),
            3 => Triple.attack(),
            4 => Quad.attack(),
            _ => panic!(),
        };

        self.matrix = matrix;
        self
    }


    pub fn display(&self) {
        println!("====================");
        let mut matrix = self.matrix;

        for (y, x) in self.current_piece.absolute() {
            matrix[y as usize][x as usize] = true;
        }

        display_matrix(&matrix);
        println!("====================");
    }
}
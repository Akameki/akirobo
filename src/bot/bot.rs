//! Defines and implements a Botris bot, currently responsible for sending a move to the server given a game state.

use crate::botris::{api_messages::{ActionPayload, RequestMovePayload}, types::GameState};
use crate::botris::types::Command::*;
use super::matrix::*;

use rand::Rng;

// pub trait Respond {
//     fn request_moves(event: &RequestMovePayload) -> ActionPayload;
// }

pub struct Bot {
    board: [[bool; 10]; 20],
    bag: u32,
    piece: u32,
}

impl Bot {
    pub fn new () -> Self {
        Self::default()
    }

    pub fn default() -> Self {
        Bot {
            board: [[false; 10]; 20],
            bag: 0,
            piece: 0,
        }
    }

    pub async fn request_moves(&mut self, game_state: &GameState) -> ActionPayload {
        // print board dimensions
        // if game_state.board.len() == 0 {
        //     println!("Board is empty");
        // } else {
        //     println!("Board: {}x{}", game_state.board.len(), game_state.board[0].len());
        // }
        
        let board = to_board(&game_state.board);
        display_board(&board);
        // nothing .
        // wait one second
        // then return empty action
        // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let mut action = ActionPayload::new();
        // action.append(&mut vec![Command::SonicLeft, Command::RotateCcw, Command::SonicLeft, Command::RotateCcw, Command::SonicLeft]);
        // append 5 random commands
        for _ in 0..5 {
            let command = match rand::thread_rng().gen_range(0..=6) {
                0 => SonicLeft,
                1 => SonicRight,
                2 => RotateCcw,
                3 => RotateCw,
                4 => MoveLeft,
                5 => MoveRight,
                _ => Hold,

            };
            action.push(command);
        }
        action
    }
}
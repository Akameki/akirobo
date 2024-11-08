//! Defines and implements a Botris bot, currently responsible for sending a move to the server given a game state.
//! Traverses possible moves and calling eval on each possible board state.
use std::i32::MIN;

use crate::{botris::{api_messages::{ActionPayload, RequestMovePayload}, game_info::BOARD_HEIGHT, types::GameState}, game::{frame::Frame, eval::{self, eval}}};
use crate::botris::types::Command;
use crate::botris::types::Command::*;
use crate::game::matrix::*;

use rand::Rng;

pub struct Bot {
    board: [[bool; 10]; BOARD_HEIGHT],
    bag: u32,
    piece: u32,
}

impl Bot {
    pub fn new () -> Self {
        Bot {
            board: EMPTY_BOARD,
            bag: 0,
            piece: 0,
        }
    }

    pub async fn request_moves(&mut self, game_state: &GameState) -> ActionPayload {
        
        let engine = Frame::new(game_state);
        engine.display();

        // generate moves by traversing all rotations followed by 0-5 moves.
        let mut commands: Vec<Command> = Vec::new();
        let mut best_score = MIN;
        let mut best_commands: Vec<Command> = Vec::new();
        for rotation in 0..4 {
            for moves in 0..6 {
                for direction in 0..2 {
                    commands.clear();
                    
                    for _ in 0..rotation {
                        commands.push(RotateCw);
                    }
                    for _ in 0..moves {
                        commands.push(if direction == 0 {MoveLeft} else {MoveRight});
                    }
                    let mut engine2 = engine.clone();
                    engine2.run_commands(commands.clone()).hard_drop();
                    let score = eval(&engine2);
                    
                    // engine2.display();
                    if score > best_score {
                        println!("Improvement!");
                        best_score = score;
                        best_commands = commands.clone();
                    }
                    println!("~~~~~~~");
                    println!("Score: {} with commands {:?}", score, commands);
                }

            }
        }
        println!("Best score {} with commands {:?}", best_score, best_commands);
        println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        ActionPayload::new(best_commands)
    }

    // RAND bot.
    // pub async fn request_moves(&mut self, game_state: &GameState) -> ActionPayload {
        
    //     let engine = Engine::new(game_state);
    //     engine.display();

    //     // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //     let mut action = ActionPayload::new();
    //     for _ in 0..5 {
    //         let command = match rand::thread_rng().gen_range(0..=6) {
    //             0 => SonicLeft,
    //             1 => SonicRight,
    //             2 => RotateCcw,
    //             3 => RotateCw,
    //             4 => MoveLeft,
    //             5 => MoveRight,
    //             _ => Hold,

    //         };
    //         action.push(command);
    //     }
    //     action
    // }

    // LEFT bot.
    // pub async fn request_moves(&mut self, game_state: &GameState) -> ActionPayload {
    //     let original_engine = Engine::new(game_state);
    //     original_engine.display();

    //     let mut action = ActionPayload::new();
    //     action.push(RotateCcw);
    //     action.push(MoveLeft);
    //     action.push(MoveLeft);
    //     action.push(MoveLeft);
    //     action.push(MoveLeft);
    //     action.push(MoveLeft);

    //     let mut engine = original_engine.clone();
    //     engine.try_commands(&action.commands);
    //     engine.display();
    //     let score = eval(&engine.matrix);
    //     println!("Score: {}", score);
    //     println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    //     action
    // }
}
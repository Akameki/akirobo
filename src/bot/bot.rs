//! Defines and implements a Botris bot, currently responsible for sending a move to the server given a game state.
//! Traverses possible moves and calling eval on each possible board state.

use crate::{botris::{api_messages::{ActionPayload, RequestMovePayload}, game_info::BOARD_HEIGHT, types::GameState}, game::{frame::Frame, eval::Evaluator}};
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
        println!("\n\n\n");
        // println!("bag: {}, queue: {}", game_state.bag, game_state.queue);
        
        let mut engine = Frame::new(game_state);
        engine.display();

        // generate moves by traversing all rotations followed by 0-5 moves.
        let mut commands: Vec<Command> = Vec::new();
        let mut best_score = f64::MIN;
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
                    engine2.run_commands(commands.clone()).place();
                    let score = Evaluator::new(engine2).eval(false);
                    
                    // engine2.display();
                    if score > best_score {
                        println!("Improvement!");
                        best_score = score;
                        best_commands = commands.clone();
                    }
                }

            }
        }
        
        engine.run_commands(best_commands.clone()).place().display();
        println!("Best score {} with commands {:?}", best_score, best_commands);
        Evaluator::new(engine).eval(true);


        // tokio::time::sleep(tokio::time::Duration::from_millis(700)).await;


        ActionPayload::new(best_commands)
    }
}
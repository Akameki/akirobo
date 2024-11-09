//! Defines and implements a Botris bot, currently responsible for sending a move to the server given a game state.
//! Traverses possible moves and calling eval on each possible board state.

use std::collections::HashMap;

use crate::bot::eval::Evaluator;

use crate::botris::{api_messages::ActionPayload, types::GameState};
use crate::botris::types::Command;
use crate::botris::types::Command::*;

use crate::game::frame::Frame;

// use rand::Rng;

pub const DEPTH: usize = 3;

pub struct Bot;

impl Bot {
    pub fn new () -> Self {
        Bot
    }

    pub async fn suggest_action(&mut self, game_state: &GameState) -> ActionPayload {
        println!("\n\n\n");
        println!("bag: {:?}, queue: {:?}", game_state.bag, game_state.queue);
        
        // generate and evaluate possible moves
        let frame = Frame::from_state(game_state);
        frame.display();
        let to_evaluate = self.traverse_simple_moves(&frame);
        let mut best_score = f64::NEG_INFINITY;
        let mut best_commands = vec![];
        let mut best_frame = frame.clone();
        println!("traversing {} actions", to_evaluate.len());

        for (frame, commands) in to_evaluate {
            let score = Evaluator::new(frame.clone()).eval(false);
            if score > best_score {
                best_score = score;
                best_commands = commands.clone();
                best_frame = frame.clone();
            }
            // println!("{} {:?}", score, commands);
            
        }
        best_frame.display();
        println!("Best: {} {:?}", best_score, best_commands);
        Evaluator::new(best_frame).eval(true);


        // tokio::time::sleep(tokio::time::Duration::from_millis(700)).await;


        ActionPayload::new(best_commands)
    }

    // generates possible frames by moving all rotations of the current piece along the row
    // returns a hashmap of the frames mapped to the commands that generated them
    fn traverse_simple_moves(&self, frame: &Frame) -> HashMap<Frame, Vec<Command>> {
        let mut frames = HashMap::new();
        let mut commands: Vec<Command>;

        for rotations in 0..4 {
            if let Some(rotated_frame) = frame.run_commands(&vec![RotateCw; rotations]) {
                commands = vec![RotateCw; rotations];
                frames.insert(rotated_frame.hard_drop(), commands.clone()); // insert first for 0 moves
                let mut prev_frame = rotated_frame.clone();
                while let Some(curr_frame) = prev_frame.run_command(&MoveLeft) {
                    commands.push(MoveLeft);
                    prev_frame = curr_frame.clone();
                    insert_and_update(&mut frames, curr_frame.hard_drop(), commands.clone());
                }
                commands = vec![RotateCw; rotations];
                let mut prev_frame = rotated_frame.clone();
                while let Some(curr_frame) = prev_frame.run_command(&MoveRight) {
                    commands.push(MoveRight);
                    prev_frame = curr_frame.clone();
                    insert_and_update(&mut frames, curr_frame.hard_drop(), commands.clone());
                }
            }
        }
        frames
    }

    
}

// insert a frame if not present, or update the commands if uses fewer commands
fn insert_and_update(frames: &mut HashMap<Frame, Vec<Command>>, frame: Frame, commands: Vec<Command>) {
    if let Some(existing_commands) = frames.get(&frame) {
        if commands.len() < existing_commands.len() {
            frames.insert(frame, commands);
        }
    } else {
        frames.insert(frame, commands);
    }
}
//! Defines and implements a Botris bot, currently responsible for sending a move to the server given a game state.
//! Traverses possible moves and calling eval on each possible board state.

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

use crate::bot::eval::Evaluator;

use crate::botris::{api_messages::ActionPayload, types::GameState};
use crate::botris::types::Command;
use crate::botris::types::Command::*;

use crate::game::frame::Frame;

// use rand::Rng;

pub const DEPTH: usize = 4;

pub struct Bot {
    first_mode: bool, // true if on first depth
    first_actions: HashMap<Frame, Vec<Command>>,
    dfs_stack: Vec<Frame>,
    dfs_marked: HashSet<Frame>,

    to_evaluate: Vec<(Frame, Vec<Command>)>,
}

impl Bot {
    pub fn new() -> Self {
        Bot {
            first_mode: true,
            first_actions: HashMap::new(),
            dfs_stack: vec![],
            dfs_marked: HashSet::new(),

            to_evaluate: Vec::new(),
        }
    }

    pub async fn suggest_action(&mut self, game_state: &GameState) -> ActionPayload {
        println!("current: {:?}\nqueue: {:?}\nbag: {:?}", game_state.current.piece, game_state.queue, game_state.bag);
        
        let frame = Frame::from_state(game_state);
        frame.display();

        // generate possible frames

        self.first_actions = HashMap::new();
        self.to_evaluate = Vec::new();
        self.flood_piece(&frame);

        self.first_mode = false;

        println!("# first actions: {}", self.first_actions.len());

        // DFS
        let first_actions  = self.first_actions.clone();
        for (frame, first_commands) in first_actions {
            self.dfs_stack = vec![];
            self.dfs_marked = HashSet::new();
            self.dfs_stack.push(frame.clone());
            
            while let Some(curr_frame) = self.dfs_stack.pop() {
                if curr_frame.depth == DEPTH {
                    self.to_evaluate.push((curr_frame.clone(), first_commands.clone()));
                    continue;
                }
                self.flood_piece(&curr_frame);
            }
        }

        // evaluate possible moves
        let mut best_frame = frame.clone();
        let mut best_frame_score = f64::NEG_INFINITY;
        let mut best_frame_commands = vec![];
        println!("traversing {} end frames", &self.to_evaluate.len());

        for (frame, commands) in &self.to_evaluate {
            let score = Evaluator::new(frame.clone()).eval(false);
            if score > best_frame_score {
                best_frame = frame.clone();
                best_frame_score = score;
                best_frame_commands = commands.clone();
            }
            // println!("{} {:?}", score, commands);
            
        }
        best_frame.display();
        println!("Best: {} {:?}", best_frame_score, best_frame_commands);
        Evaluator::new(best_frame).eval(true);


        // tokio::time::sleep(tokio::time::Duration::from_millis(700)).await;


        ActionPayload::new(best_frame_commands)
    }


    // insert a frame if not present, or update the commands if uses fewer commands
    fn insert_maybe_update(&mut self, frame: Frame, commands: Vec<Command>) {
        if self.first_mode {
            if let Some(existing_commands) = self.first_actions.get(&frame) {
                if commands.len() < existing_commands.len() {
                    self.first_actions.insert(frame, commands);
                }
            } else {
                self.first_actions.insert(frame, commands);
            }
        } else if !self.dfs_marked.contains(&frame) {
                self.dfs_marked.insert(frame.clone());
                self.dfs_stack.push(frame.clone());
        
        }
    }

    // generates possible frames by moving all rotations of the current piece along the row
    // returns a hashmap of the frames mapped to the commands that generated them
    fn flood_piece(&mut self, frame: &Frame) {

        let mut commands: Vec<Command>;

        for rotations in 0..2 {
            if let Some(rotated_frame) = frame.run_commands(&vec![RotateCw; rotations]) {
                commands = vec![RotateCw; rotations];
                self.insert_maybe_update(rotated_frame.hard_drop(), commands.clone()); // insert first for 0 moves
                self.flood_x(&rotated_frame, MoveLeft, commands.clone());
                self.flood_x(&rotated_frame, MoveRight, commands.clone());
            } else {
                break;
            }
        }
        if let Some(rotated_frame) = frame.run_command(&RotateCcw) {
            commands = vec![RotateCcw];
            self.insert_maybe_update(rotated_frame.hard_drop(), commands.clone());
            self.flood_x(&rotated_frame, MoveLeft, commands.clone());
            self.flood_x(&rotated_frame, MoveRight, commands.clone());
        }

    }

    fn flood_x(&mut self, frame: &Frame, command: Command, mut commands: Vec<Command>) {
            let mut prev_frame = frame.clone();
            while let Some(curr_frame) = prev_frame.run_command(&command) {
                commands.push(command);
                prev_frame = curr_frame.clone();
                self.insert_maybe_update(curr_frame.hard_drop(), commands.clone());
            }
        }
    }

    // for dfs
    // fn generate_depth(&self, frame: &Frame, queue: &mut VecDeque<Frame>, ) {
    //     let mut commands: Vec<Command>;

    //     for rotations in 0..4 {
    //         if let Some(rotated_frame) = frame.run_commands(&vec![RotateCw; rotations]) {
    //             commands = vec![RotateCw; rotations];
    //             queue.push_back(rotated_frame.hard_drop()); // insert first for 0 moves
    //             let mut prev_frame = rotated_frame.clone();
    //             while let Some(curr_frame) = prev_frame.run_command(&MoveLeft) {
    //                 queue.push_back(curr_frame.hard_drop());
    //                 prev_frame = curr_frame.clone();
    //             }
    //             let mut prev_frame = rotated_frame.clone();
    //             while let Some(curr_frame) = prev_frame.run_command(&MoveRight) {
    //                 queue.push_back(curr_frame.hard_drop());
    //                 prev_frame = curr_frame.clone();
    //             }
    //         }
    //     }
    // }



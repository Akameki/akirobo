//! Defines and implements a Botris bot, currently responsible for sending a move to the server given a game state.
//! Traverses possible moves and calling eval on each possible board state.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::bot::eval::Evaluator;

use crate::botris::{api_messages::ActionPayload, types::GameState};
use crate::botris::types::Command;
use crate::botris::types::Command::*;

use crate::game::frame::Frame;

// use rand::Rng;

pub const QUEUE_DEPTH: usize = 5; // 1 - 7
pub const PRUNE_TOP_N: usize = 3;


pub struct Bot {
    first_actions: HashMap<Frame, Vec<Command>>,
    dfs_stack: Vec<Frame>,
    dfs_marked: HashSet<Frame>,

    to_evaluate: Vec<(Frame, Vec<Command>)>,
}

impl Bot {
    pub fn new() -> Self {
        Bot {
            first_actions: HashMap::new(),
            dfs_stack: vec![],
            dfs_marked: HashSet::new(),

            to_evaluate: Vec::new(),
        }
    }

    pub async fn suggest_action(&mut self, game_state: &GameState) -> ActionPayload {
        // if first piece, hold
        if game_state.held.is_none() {
            return ActionPayload::new(vec![Hold]);
        }
        println!("current: {:?}\nqueue: {:?}\nbag: {:?}", game_state.current.piece, game_state.queue, game_state.bag);
        
        let frame = Frame::from_state(game_state);
        frame.display();

        // generate possible frames

        self.first_actions = HashMap::new();
        self.to_evaluate = Vec::new();
        self.generate_placements(&frame, true);


        println!("# first actions: {}", self.first_actions.len());

        // DFS from each first action.
        let first_actions  = self.first_actions.clone();
        for (frame, first_commands) in first_actions {
            self.dfs_stack = Vec::new();
            self.dfs_marked = HashSet::new();
            self.dfs_stack.push(frame.clone());
            
            while let Some(curr_frame) = self.dfs_stack.pop() {
                if curr_frame.depth == QUEUE_DEPTH {
                    self.to_evaluate.push((curr_frame.clone(), first_commands.clone()));
                    continue;
                }
                let placements = self.generate_placements(&curr_frame, false);
                // evaluate each placement and only keep top 20
                let top_placements = self.filter_top_n_placements(placements, PRUNE_TOP_N);
                for placement in top_placements {
                    if !self.dfs_marked.contains(&placement) {
                        self.dfs_marked.insert(placement.clone());
                        self.dfs_stack.push(placement);
                    }
                }
            }
        }

        


        // evaluate possible moves
        let mut best_frame = frame.clone();
        let mut best_frame_score = f64::NEG_INFINITY;
        let mut best_frame_commands = vec![];
        println!("generated {} end frames\nevaluation time", &self.to_evaluate.len());

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
        println!("Best: {} {:?} (depth {})", best_frame_score, best_frame_commands, QUEUE_DEPTH);
        Evaluator::new(best_frame).eval(true);


        // tokio::time::sleep(tokio::time::Duration::from_millis(700)).await;


        ActionPayload::new(best_frame_commands)
    }

    // insert a frame if not present, or update the commands if uses fewer commands
    fn insert_least_complex(&mut self, frame: Frame, commands: Vec<Command>) {
        if let Some(existing_commands) = self.first_actions.get(&frame) {
            if commands.len() < existing_commands.len() {
                self.first_actions.insert(frame, commands);
            }
        } else {
            self.first_actions.insert(frame, commands);
        }
    }

    // generates possible frames by moving all rotations of the current piece along the row
    // returns a hashmap of the frames mapped to the commands that generated them
    fn generate_placements(&mut self, frame: &Frame, depth_one: bool) -> HashSet<Frame> {
        let mut possible_placements = HashSet::new();
 
        let premoves = [
            vec![], vec![Hold],
            vec![RotateCw], vec![RotateCw, RotateCw], vec![Hold, RotateCw], vec![Hold, RotateCw, RotateCw],
            vec![RotateCcw], vec![Hold, RotateCcw],
        ];

        // if depth one, only worry about populating the first_actions hashmap

        for premove in premoves {
            if let Some(premoved_frame) = frame.run_commands(&premove) {
                if depth_one {
                    self.flood_first_actions_in_direction(premoved_frame.clone(), MoveLeft, premove.clone());
                    self.flood_first_actions_in_direction(premoved_frame.clone(), MoveRight, premove.clone());
                } else {
                    self.flood_placements_in_direction(premoved_frame.clone(), MoveLeft, &mut possible_placements);
                    self.flood_placements_in_direction(premoved_frame.clone(), MoveRight, &mut possible_placements);
                }
            }
        }
        possible_placements
    }
    fn flood_first_actions_in_direction(&mut self, promoved_frame: Frame, move_dir: Command, mut premoves: Vec<Command>) {
        let mut maybe_frame = Some(promoved_frame);
        while let Some(curr_frame) = maybe_frame {
            self.insert_least_complex(curr_frame.hard_drop(), premoves.clone());
            premoves.push(move_dir);
            maybe_frame = curr_frame.run_command(&move_dir);
        }
    }

    fn flood_placements_in_direction(&mut self, promoved_frame: Frame, move_dir: Command, possible_placements: &mut HashSet<Frame>) {
        let mut maybe_frame = Some(promoved_frame);
        while let Some(curr_frame) = maybe_frame {
            possible_placements.insert(curr_frame.hard_drop());
            maybe_frame = curr_frame.run_command(&move_dir);
        }
    }

    fn filter_top_n_placements(&self, placements: HashSet<Frame>, n: usize) -> Vec<Frame> {
        let mut sorted_placements = placements.iter().map(|frame| {
            let score = Evaluator::new(frame.clone()).eval(false);
            (frame.clone(), score)
        }).collect::<Vec<_>>();
        sorted_placements.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted_placements.iter().map(|(frame, _)| frame.clone()).take(n).collect()
    }

}
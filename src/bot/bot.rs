//! Defines and implements a Botris bot, currently responsible for sending a move to the server given a game state.
//! Traverses possible moves and calling eval on each possible board state.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::bot::eval::Evaluator;

use crate::botris::{api_messages::ActionPayload, types::GameState};
use crate::botris::types::Command;
use crate::botris::types::Command::*;

use crate::game::frame::Frame;

// use rand::Rng;

pub const QUEUE_DEPTH: usize = 1; // 1-7
pub const PRUNE_TOP_N: usize = 0; // 0 = no pruning


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

            // mid_seen: HashSet::new(),

            to_evaluate: Vec::new(),
        }
    }

    pub async fn suggest_action(&mut self, game_state: &GameState) -> ActionPayload {
        // if first piece, hold
        if game_state.held.is_none() {
            return ActionPayload::new(vec![Hold]);
        }
        // println!("current: {:?}\nqueue: {:?}\nbag: {:?}", game_state.current.piece, game_state.queue, game_state.bag);
        
        let frame = Frame::from_state(game_state);
        frame.display();

        /* generate possible frames */

        self.first_actions = HashMap::new();
        self.to_evaluate = Vec::new();
        self.generate_placements(&frame, true);

        // self.move_gen(&frame, true, vec![], &mut HashSet::new());
        // self.move_gen(&frame.run_command(&Hold), true, vec![Hold], &mut HashSet::new());


        println!("# first actions: {}", self.first_actions.len());

        let mut best: (Frame, Vec<Command>) = (frame.clone(), vec![]);
        let mut best_score = f64::NEG_INFINITY;

        // assume depth = 1 for now
        for (first_action, first_commands) in &self.first_actions {
            let hd = first_action.hard_drop();
            self.to_evaluate.push((hd.clone(), first_commands.clone()));
        }

        // // DFS from each first action.
        // let first_actions  = self.first_actions.clone();
        // for (frame, first_commands) in first_actions {
        //     self.dfs_stack = Vec::new();
        //     self.dfs_marked = HashSet::new();
        //     self.dfs_stack.push(frame.clone());
            
        //     while let Some(curr_frame) = self.dfs_stack.pop() {
        //         if curr_frame.depth == QUEUE_DEPTH {
        //             self.to_evaluate.push((curr_frame.clone(), first_commands.clone()));
        //             continue;
        //         }
        //         let placements = self.generate_placements(&curr_frame, false);
        //         // evaluate each placement and only keep top 20
        //         let top_placements = self.filter_top_n_placements(placements, PRUNE_TOP_N);
        //         for placement in top_placements {
        //             if !self.dfs_marked.contains(&placement) {
        //                 self.dfs_marked.insert(placement.clone());
        //                 self.dfs_stack.push(placement);
        //             }
        //         }
        //     }
        // }

        


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
            if let Some(premoved_frame) = frame.try_commands(&premove) {
                self.flood_in_direction(premoved_frame.clone(), MoveLeft, premove.clone(), &mut possible_placements, depth_one);
                self.flood_in_direction(premoved_frame.clone(), MoveRight, premove.clone(), &mut possible_placements, depth_one);
            }
        }
        possible_placements
    }

    // recursively find placements
    fn generate_placements2(&mut self, frame: &Frame, depth_one: bool, commands: Vec<Command>, seen: &mut HashSet<Frame>, toggle: usize) -> HashSet<Frame> {
  

        let mut possible_placements = HashSet::new();

        if toggle > 5 {
            return possible_placements;
        }
 
        // try hold:
        if frame.can_hold {
            let mut cmds = commands.clone();
            cmds.push(Hold);
            if let Some(new_frame) = frame.try_command(&Hold) {
                if !seen.contains(&new_frame) {
                    seen.insert(new_frame.clone());
                    self.generate_placements2(&new_frame, depth_one, cmds.clone(), seen, toggle);
                }
            }
        }

        let moves = [[MoveLeft, MoveRight], [RotateCw, RotateCcw]];


        for move_dir in moves[toggle % 2] {
            if let Some(new_frame) = frame.try_command(&move_dir) {
                if !seen.contains(&new_frame) {
                    seen.insert(new_frame.clone());
                    let mut commands2 = commands.clone();
                    commands2.push(move_dir);
                    possible_placements.extend(self.generate_placements2(&new_frame, depth_one, commands2.clone(), seen, toggle));
                    // sonic drop and toggle
                    let mut sonicced = new_frame.clone();
                    if let Some(new_frame) = new_frame.try_command(&SonicDrop) {
                        sonicced = new_frame;
                    }
                    if !seen.contains(&sonicced) {
                        seen.insert(sonicced.clone());
                        let mut commands2 = commands.clone();
                        commands2.push(SonicDrop);
                        possible_placements.extend(self.generate_placements2(&sonicced, depth_one, commands2.clone(), seen, toggle + 1));
                        if depth_one {
                            self.insert_least_complex(sonicced.hard_drop(), commands2);
                        } else {
                            possible_placements.insert(sonicced.hard_drop());
                        }
                    }
                }
            }
        }
        possible_placements
    }

    fn add_if_unseen(&mut self, frame: &Frame, depth_one: bool, commands: Vec<Command>, possible_placements: &mut HashSet<Frame>) -> bool {
        let sonic = frame.force_sonic_drop();
        let mut commands2 = commands.clone();
        commands2.push(SonicDrop);
        if !possible_placements.contains(&sonic) {
            possible_placements.insert(sonic.clone());
            if depth_one {
                self.insert_least_complex(sonic.clone(), commands2.clone());
            }
            self.move_gen(&sonic, depth_one, commands2.clone(), possible_placements);
            true
        } else {
            false
        }
    }

    // rotate, move, softdrop. repeat
    // a states are considered the same if they are the same after a hard drop
    fn move_gen(&mut self, original_frame: &Frame, depth_one: bool, commands: Vec<Command>, possible_placements: &mut HashSet<Frame>) {

        let rotation_set = [RotateCw, RotateCcw];
        let move_set = [MoveLeft, MoveRight];

        for rotate_command in rotation_set {
            let mut rotating_frame = original_frame.clone();
            let mut rotating_commands = commands.clone();
            for _ in 0..=2 {
                self.add_if_unseen(&rotating_frame, depth_one, rotating_commands.clone(), possible_placements);
                for move_command in move_set {
                    let mut moving_frame = rotating_frame.clone();
                    let mut moving_commands = rotating_commands.clone();
                    for _ in 0..=9 {
                        self.add_if_unseen(&moving_frame, depth_one, moving_commands.clone(), possible_placements);
                        moving_frame = moving_frame.run_command(&move_command);
                        moving_commands.push(move_command);
                    }
                }
                rotating_frame = rotating_frame.run_command(&rotate_command);
                rotating_commands.push(rotate_command);
            }
            
        }
    }

    // fn add_to_bfs_queue
    fn move_gen_bfs(&mut self, original_frame: &Frame, depth_one: bool, commands: Vec<Command>, possible_placements: &mut HashSet<Frame>) {
        let mut queue = VecDeque::new();
        queue.push_back((original_frame.clone(), commands.clone()));

    }

    fn flood_in_direction(&mut self, promoved_frame: Frame, move_dir: Command, mut premoves: Vec<Command>, possible_placements: &mut HashSet<Frame>, depth_one: bool) {
        let mut maybe_frame = Some(promoved_frame);
        while let Some(curr_frame) = maybe_frame {
            // try sonicdrop and rotations:
            for postmoves in [vec![], vec![SonicDrop, RotateCw], vec![SonicDrop, RotateCcw]] {
                let mut cmds = premoves.clone();
                cmds.extend(postmoves.clone());
                if let Some(new_frame) = curr_frame.try_commands(&postmoves) {
                    if depth_one {
                        self.insert_least_complex(new_frame.hard_drop(), cmds);
                    } else {
                        possible_placements.insert(new_frame.hard_drop());
                    }
                }
            }
            premoves.push(move_dir);
            maybe_frame = curr_frame.try_command(&move_dir);
        }
    }

    fn filter_top_n_placements(&self, placements: HashSet<Frame>, n: usize) -> Vec<Frame> {

        if n == 0 { // don't filter
            return placements.iter().map(|frame| frame.clone()).collect();
        }

        let mut sorted_placements = placements.iter().map(|frame| {
            let score = Evaluator::new(frame.clone()).eval(false);
            (frame.clone(), score)
        }).collect::<Vec<_>>();
        sorted_placements.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sorted_placements.iter().map(|(frame, _)| frame.clone()).take(n).collect()
    }

}
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    time::Instant,
};

use owo_colors::OwoColorize;



use crate::{
    bot::{eval::Evaluator, eval_frames::{keep_top_frames, ScoredFrame}},
    botris::types::{Command, GameState},
    game::frame::Frame,
};

const LOOKAHEAD_DEPTH: usize = 25;
const BRANCHING_FACTOR: usize = 6;

#[derive(Default)]
pub struct Akirobo {
    seen_frames: HashSet<Frame>,
    total_generated_actions: usize,
    total_evaluated: usize,
}

impl Akirobo {
    pub fn new() -> Self {
        Akirobo { seen_frames: HashSet::new(), total_generated_actions: 0, total_evaluated: 0 }
    }

    pub async fn suggest_action(&mut self, game_state: &GameState) -> Vec<Command> {
        // return vec![Command::RotateCcw, Command::RotateCcw, Command::RotateCcw, Command::RotateCcw];
        if game_state.held.is_none() {
            return vec![Command::Hold];
        }
        let start_time = Instant::now();
        let starting_frame = Frame::from_state(game_state);

        let mut curr_moves = self.move_gen(starting_frame.clone(), None);
        let mut prev_moves;
        for _ in 0..LOOKAHEAD_DEPTH {
            self.total_generated_actions += curr_moves.len();
            prev_moves = curr_moves;
            curr_moves = HashMap::new();
            if BRANCHING_FACTOR == 0 {
                // keep ALL moves
                for (frame, first_action) in prev_moves {
                    curr_moves.extend(self.move_gen(frame, Some(first_action)));
                }
            } else {
                // evaluate and keep top BRANCHING_FACTOR moves per depth level.
                self.total_evaluated += prev_moves.len();
                let top_moves = keep_top_frames(prev_moves, BRANCHING_FACTOR);
                for ScoredFrame { frame, first_action, score: _ } in top_moves {
                    curr_moves.extend(self.move_gen(frame, Some(first_action)));
                }
            }
        }
        self.total_generated_actions += curr_moves.len();
        let mut best_action = Rc::new(Vec::new());
        let mut best_eval = f64::NEG_INFINITY;
        for (frame, action) in curr_moves {
            let eval = Evaluator::new(frame, false).eval();
            if eval > best_eval {
                best_action = action;
                best_eval = eval;
            }
        }
        let mut suggested_next_frame = starting_frame;
        for command in best_action.iter() {
            suggested_next_frame = suggested_next_frame.run_command(*command);
        }
        println!("{}", suggested_next_frame.force_sonic_drop().green());
        println!("{:?}", best_action);
        let millis = start_time.elapsed().as_millis();
        Evaluator::new(suggested_next_frame.hard_drop(), true).eval();
        println!(
            "generated {} moves in {}ms ({} pps)",
            self.total_generated_actions.blue(),
            millis.blue(),
            1000.0 / millis as f64
        );
        best_action.as_ref().clone()
    }

    fn move_gen(
        &mut self,
        original_frame: Frame,
        first_action: Option<Rc<Vec<Command>>>,
    ) -> HashMap<Frame, Rc<Vec<Command>>> {
        use Command::*;
        let mut generated = HashMap::new();
        let rotation_sets = [
            vec![],
            vec![RotateCw],
            vec![RotateCcw],
            vec![RotateCcw, RotateCcw],
            vec![Hold],
            vec![Hold, RotateCw],
            vec![Hold, RotateCcw],
            vec![Hold, RotateCcw, RotateCcw],
        ];
        if !self.seen_frames.insert(original_frame.clone()) {
            return generated;
        }

        for rotation_set in rotation_sets {
            let frame = original_frame.clone();
            if let Some(rotated) = frame.try_commands(&rotation_set) {
                generated.insert(
                    rotated.hard_drop(),
                    first_action.clone().unwrap_or(Rc::new(rotation_set.clone())),
                );
                for dir in [MoveLeft, MoveRight] {
                    let mut nudged_actions = rotation_set.clone();
                    let mut nudging = rotated.clone();
                    for _ in 0..6 {
                        if let Some(nudged) = nudging.clone().try_command(dir) {
                            nudged_actions.push(dir);
                            generated.insert(
                                nudged.hard_drop(),
                                first_action.clone().unwrap_or(Rc::new(nudged_actions.clone())),
                            );
                            nudging = nudged;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        generated
    }
}

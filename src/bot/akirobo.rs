use std::{
    array,
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
    time::Instant,
};

use owo_colors::OwoColorize;

use super::evaluation::Evaluate;
use crate::{
    bot::{evaluation::default_eval::DefaultEval, searchtree::PlacementNode},
    botris::types::Command,
    game::frame::Frame,
};

const LOOKAHEAD_DEPTH: usize = 7; // # pieces in queue being considered (0 = only current)
const DEPTH_ZERO_SIZE: usize = 10;
const BRANCHING_FACTOR: usize = 3; // not actually branching. # every depth of "tree".

#[derive(Default)]
pub struct Akirobo {
    total_generated_placements: usize,
}

impl Akirobo {
    pub fn new() -> Self {
        Akirobo {
            total_generated_placements: 0,
        }
    }

    pub fn suggest_action(&mut self, genesis: Frame) -> Vec<Command> {
        let start_time = Instant::now();
        let evaluator = DefaultEval {};

        let action_lookup = self.move_gen_with_action(&genesis);

        let mut tree_nodes: [BTreeSet<Rc<PlacementNode>>; LOOKAHEAD_DEPTH + 1] =
            array::from_fn(|_| BTreeSet::new());

        for placement in action_lookup.keys() {
            let after_lock = placement.lock_piece();
            tree_nodes[0].insert(PlacementNode::new(placement, &after_lock, None, &evaluator));
        }

        // for each node in previous depth, add BRANCHING_FACTOR new nodes.
        for depth in 1..=LOOKAHEAD_DEPTH {
            let (before, after) = tree_nodes.split_at_mut(depth);
            let prev_depth_nodes = &before[depth - 1];
            let curr_depth_nodes = &mut after[0]; // starts empty
            let filtered = prev_depth_nodes.iter().rev().take(if depth == 1 {
                DEPTH_ZERO_SIZE
            } else {
                BRANCHING_FACTOR * depth
            });
            for node in filtered {
                let mut children = BTreeSet::new();
                for placement in self.move_gen(&node.after_lock) {
                    let after_lock = placement.lock_piece();
                    children.insert(PlacementNode::new(
                        &placement,
                        &after_lock,
                        Some(node.clone()),
                        &evaluator,
                    ));
                }
                curr_depth_nodes.append(&mut children);
            }
        }

        let millis = start_time.elapsed().as_millis();
        let total_resulting_frames = tree_nodes.last().unwrap().len();

        let best_node = tree_nodes.last_mut().unwrap().pop_last().unwrap();
        let path_to_node = best_node.get_path_from_root();
        let suggestion = action_lookup.get(&path_to_node[0]).unwrap().to_owned();

        Frame::print_frames(&[path_to_node[0].clone(), path_to_node.last().unwrap().clone()]);
        println!("Suggestion: {:?}", suggestion);
        evaluator.eval(&path_to_node[0].lock_piece(), true);
        println!(
            "\"{}\" final frames in {}ms ({:.2}pps)\nGenerated {} total placements\n",
            total_resulting_frames.blue(),
            millis.blue(),
            1000.0 / millis as f32,
            self.total_generated_placements,
        );

        suggestion
    }

    /// Generates possible placements for current piece (DOES NOT ADVANCE PIECE)
    fn move_gen_with_action(&mut self, gen_from: &Frame) -> HashMap<Frame, Vec<Command>> {
        use Command::*;
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


        let mut all_rotations_and_holds = HashMap::new();
        for rotation_set in rotation_sets {
            if let Some(rotated_frame) = gen_from.try_commands(&rotation_set) {
                all_rotations_and_holds.insert(rotated_frame, rotation_set);
            }
        }
        let mut translated_and_soniced = HashMap::new();
        for (frame, action) in all_rotations_and_holds {
            for direction in [MoveLeft, MoveRight] {
                let mut translating_frame = frame.clone();
                let mut translating_action = action.clone();
                while let Some(translated_more) = translating_frame.try_command(direction) {
                    translating_action.push(direction);
                    translating_frame = translated_more;
                    translated_and_soniced
                        .insert(translating_frame.force_sonic_drop(), translating_action.clone());
                }
            }
            translated_and_soniced.insert(frame.force_sonic_drop(), action);
        }

        let mut generated = translated_and_soniced.clone();

        for (frame, action) in translated_and_soniced {
            for spin in [RotateCcw, RotateCw] {
                if let Some(mut spun) = frame.try_command(spin) {
                    spun = spun.force_sonic_drop();
                    generated.entry(spun).or_insert_with(|| {
                        let mut spun_action = action.clone();
                        spun_action.push(SonicDrop); // REMEMBER this if refactored
                        spun_action.push(spin);
                        spun_action
                    });
                }
            }
        }

        generated
    }

    fn move_gen(&mut self, gen_from: &Frame) -> HashSet<Frame> {
        use Command::*;
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


        let mut all_rotations_and_holds = HashSet::new();
        for rotation_set in rotation_sets {
            if let Some(rotated_frame) = gen_from.try_commands(&rotation_set) {
                all_rotations_and_holds.insert(rotated_frame);
            }
        }
        let mut translated_and_soniced = HashSet::new();
        for frame in all_rotations_and_holds {
            for direction in [MoveLeft, MoveRight] {
                let mut translating_frame = frame.clone();
                while let Some(translated_more) = translating_frame.try_command(direction) {
                    translating_frame = translated_more;
                    translated_and_soniced
                        .insert(translating_frame.force_sonic_drop());
                }
            }
            translated_and_soniced.insert(frame.force_sonic_drop());
        }

        let mut generated = translated_and_soniced.clone();

        for frame in translated_and_soniced {
            for spin in [RotateCcw, RotateCw] {
                if let Some(mut spun) = frame.try_command(spin) {
                    spun = spun.force_sonic_drop();
                    generated.insert(spun);
                }
            }
        }

        generated
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         botris::types::Piece,
//         game::piece::FallingPiece,
//     };

//     #[test]
//     fn test_move_gen() {
//         let mut akirobo = Akirobo::new();
//         let piece = FallingPiece::new(Piece::I);
//         let frame = Frame {
//             falling_piece: todo!(),
//             queue: todo!(),
//             held: todo!(),
//             can_hold: todo!(),
//             combo: todo!(),
//             b2b: todo!(),
//             stored_attack: todo!(),
//             confirmed_on_bottom: todo!(),
//             depth: todo!(),
//             matrix: todo!(),
//         };
//         let generated = akirobo.move_gen_with_action(&frame);
//         assert!(!generated.is_empty());
//         println!("{:?}", generated);
//     }
// }

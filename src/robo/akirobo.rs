use std::{
    array,
    collections::{BTreeSet, HashMap},
    rc::Rc,
    time::Instant,
};

use owo_colors::OwoColorize;

use crate::{
    botris::types::Command,
    evaluation::{default_eval::DefaultEval, Evaluate},
    movegen::{move_gen, move_gen_with_action},
    searchtree::{print_nodes, EvaledPlacementNode},
    tetris_core::{
        engine::{print_board, BoardData},
        snapshot::GameSnapshot,
    },
};

const LOOKAHEAD_DEPTH: usize = 6; // # pieces in queue being considered (0 = only current, disables rest)
const DEPTH_ZERO_SIZE: usize = 30; // maybe small number kinda makes bot play safer?

const BRANCHING_FACTOR: usize = 0; // if nonzero, use a large search width
const MAX_SEARCH_WIDTH: usize = 500;

// expect ~ pow(BRANCHING_FACTOR, LOOKAHEAD) leaves at final depth, or MAX_SEARCH_WIDTH.

#[derive(Default)]
pub struct Akirobo {}

impl Akirobo {
    pub fn new() -> Self {
        Akirobo {}
    }

    pub fn suggest_action(&mut self, genesis: &GameSnapshot) -> Vec<Command> {
        let start_time = Instant::now();
        let evaluator = DefaultEval {};

        let genesis_board = genesis.matrix;
        let genesis_data = BoardData {
            b2b: genesis.b2b,
            combo: genesis.combo,
            cummulative_attack: 0,
            incoming: genesis.incoming_garbage,
            simulated_garbage: 0,
        };
        let first_piece = genesis.falling_piece.piece;

        let mut tree_nodes: [BTreeSet<Rc<EvaledPlacementNode>>; LOOKAHEAD_DEPTH + 1] =
            array::from_fn(|_| BTreeSet::new());

        let mut action_lookup = HashMap::new();
        for (placement, action) in move_gen_with_action(&genesis_board, first_piece) {
            tree_nodes[0].insert(EvaledPlacementNode::new(
                &genesis_board,
                placement,
                genesis.held,
                None,
                Some(genesis_data),
                &evaluator,
            ));
            action_lookup.insert(placement, action);
        }
        // TODO: definitely need some refactoring...
        for (placement, mut action) in move_gen_with_action(&genesis_board, genesis.held) {
            tree_nodes[0].insert(EvaledPlacementNode::new(
                &genesis_board,
                placement,
                first_piece,
                None,
                Some(genesis_data),
                &evaluator,
            ));
            action.insert(0, Command::Hold);
            action_lookup.insert(placement, action);
        }

        // for each node in previous depth, add BRANCHING_FACTOR new nodes.
        for depth in 1..=LOOKAHEAD_DEPTH {
            let (before, after) = tree_nodes.split_at_mut(depth);
            let prev_depth_nodes = &before[depth - 1];
            let curr_depth_nodes = &mut after[0]; // starts empty
            let filtered = match depth {
                1 => prev_depth_nodes.iter().rev().take(DEPTH_ZERO_SIZE),
                _ => prev_depth_nodes.iter().rev().take(MAX_SEARCH_WIDTH),
            };
            for node in filtered {
                let mut children = BTreeSet::new();
                for placement in move_gen(&node.board_after_clears, genesis.queue[depth - 1]) {
                    children.insert(EvaledPlacementNode::new(
                        &node.board_after_clears,
                        placement,
                        node.held,
                        Some(node.clone()),
                        None,
                        &evaluator,
                    ));
                }
                for placement in move_gen(&node.board_after_clears, node.held) {
                    children.insert(EvaledPlacementNode::new(
                        &node.board_after_clears,
                        placement,
                        genesis.queue[depth - 1],
                        Some(node.clone()),
                        None,
                        &evaluator,
                    ));
                }
                match BRANCHING_FACTOR {
                    0 => curr_depth_nodes.append(&mut children),
                    n => curr_depth_nodes.extend(children.into_iter().rev().take(n)),
                }
            }
        }

        let millis = start_time.elapsed().as_millis();
        let last_depth_frames = tree_nodes.last().unwrap().len();

        if last_depth_frames == 0 {
            print_board(&genesis_board);
            println!("We are doomed :)");
            for level in tree_nodes.iter_mut().rev() {
                if !level.is_empty() {
                    return action_lookup
                        .get(&level.pop_last().unwrap().get_root_placement())
                        .unwrap()
                        .to_owned();
                }
            }
            // death wiggle
            return vec![
                Command::SonicLeft,
                Command::SonicRight,
                Command::SonicLeft,
                Command::SonicRight,
            ];
        }

        let last_nodes = tree_nodes.last_mut().unwrap();
        let best_node = last_nodes.pop_last().unwrap();
        let best_node_root = best_node.get_root();
        let suggestion = action_lookup.get(&best_node_root.placement).unwrap().to_owned();

        
        // println!("Showing: all first moves");
        // print_nodes(tree_nodes[0].iter().rev().collect::<Vec<_>>(), 5);

        println!("Showing: best suggestion and its vision");
        let mut nodes_to_print =
            best_node.get_nodes_from_root().into_iter().take(3).collect::<Vec<_>>();
        nodes_to_print.push(best_node);
        print_nodes(&nodes_to_print, 5);

        println!("Suggestion: {:?}", suggestion);
        evaluator.eval(&best_node_root.board_after_clears, &best_node_root.board_data, true);
        println!(
            "{} placements at final depth in {}ms ({:.2}pps)",
            last_depth_frames,
            millis.blue(),
            1000.0 / millis as f32,
        );
        println!("       {}", " = ".repeat(15).black().on_bright_white());

        suggestion
    }
}

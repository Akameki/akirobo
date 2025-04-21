use std::{f32, rc::Rc};

use ordered_float::OrderedFloat;

use super::evaluation::Evaluate;
use crate::{
    botris::types::Piece,
    movegen::Placement,
    tetris_core::engine::{BitBoard, BoardData},
};

pub struct EvaledPlacementNode {
    pub board: BitBoard,
    pub placement: Placement,
    pub board_after_clears: BitBoard,
    pub board_data: BoardData,
    pub held: Piece, // not sure where this should belong yet.
    // pub cumm_attack: u32, // not sure where this should belong yet.
    pub parent: Option<Rc<EvaledPlacementNode>>,
    pub score: OrderedFloat<f32>,
    pub depth: usize,
}

impl EvaledPlacementNode {
    pub fn new(
        board: &BitBoard,
        placement: Placement,
        held: Piece,
        parent: Option<Rc<EvaledPlacementNode>>,
        board_data_if_root: Option<BoardData>,
        evaluator: &impl Evaluate,
    ) -> Rc<Self> {
        // calculate lines and clear data
        let mut filled_board = *board;
        for (y, x) in placement.piece_location {
            filled_board.set(y as usize, x as usize, true);
        }
        let data = if let Some(parent) = &parent {
            parent.board_data
        } else {
            board_data_if_root.unwrap()
        };
        let (board_after_clears, board_data) = filled_board.hard_drop(placement.all_spin, data);

        Rc::new(EvaledPlacementNode {
            score: evaluator.eval(&board_after_clears, &board_data, false),
            parent: parent.clone(),
            placement,
            held,
            board: *board,
            board_after_clears,
            board_data,
            depth: parent.as_ref().map_or(0, |p| p.depth + 1),
        })
    }

    pub fn get_root(self: &Rc<Self>) -> Rc<EvaledPlacementNode> {
        self.parent.as_ref().map_or(self.clone(), |p| p.get_root())
    }

    pub fn get_root_placement(&self) -> Placement {
        self.parent.as_ref().map_or(self.placement, |p| p.get_root_placement())
    }

    pub fn get_placements_from_root(&self) -> Vec<(BitBoard, Placement)> {
        let mut path = vec![(self.board, self.placement)];
        let mut current = self.parent.clone();
        while let Some(node) = current {
            path.push((node.board, node.placement));
            current = node.parent.clone();
        }
        path.reverse();
        path
    }

    pub fn get_nodes_from_root(self: &Rc<Self>) -> Vec<Rc<EvaledPlacementNode>> {
        let mut list = vec![self.clone()];
        let mut current = self.parent.clone();
        while let Some(node) = current {
            list.push(node.clone());
            current = node.parent.clone();
        }
        list.reverse();
        list
    }
}

impl Ord for EvaledPlacementNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}
impl PartialOrd for EvaledPlacementNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for EvaledPlacementNode {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth
            && self.placement == other.placement
            && self.board_data == other.board_data
            && self.board_after_clears == other.board_after_clears
    }
}
impl Eq for EvaledPlacementNode {}

/// print each placement in a single row
pub fn print_nodes<'a, I>(nodes: I, chunk_size: usize)
where
    I: IntoIterator<Item = &'a Rc<EvaledPlacementNode>>,
{
    let nodes: Vec<&Rc<EvaledPlacementNode>> = nodes.into_iter().collect();
    for chunk in nodes.chunks(chunk_size) {
        BitBoard::print_rows(
            &chunk
                .iter()
                .map(|node| (&node.board_after_clears, Some(node.placement.piece_location)))
                .collect::<Vec<_>>(),
            chunk_size,
        );
        for rc_node in chunk {
            print!(">     eval: {:5.1}    <", rc_node.score);
        }
        println!();
    }
}

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::rc::Rc;

use crate::botris::types::Command;
use crate::game::frame::Frame;

use super::eval::Evaluator;

// Wrapper struct for comparing frames by eval score (min-heap)
#[derive(PartialEq)]
pub struct ScoredFrame {
    pub frame: Frame,
    pub first_action: Rc<Vec<Command>>,
    pub score: f64,
}

impl Eq for ScoredFrame {}

impl PartialOrd for ScoredFrame {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredFrame {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.total_cmp(&other.score).reverse()
    }
}

pub fn keep_top_frames(frames: HashMap<Frame, Rc<Vec<Command>>>, top_n: usize) -> Vec<ScoredFrame> {
    let mut heap = BinaryHeap::with_capacity(top_n + 1);
    
    for (frame, first_action) in frames {
        let mut evaluator = Evaluator::new(frame.clone(), false);
        let score = evaluator.eval();
        heap.push(ScoredFrame { frame, first_action, score });       
    }
    
    heap.into_sorted_vec()
        .into_iter()
        .take(top_n)
        .collect()
}

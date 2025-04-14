use std::{f32, rc::Rc};

use ordered_float::OrderedFloat;

use super::evaluation::Evaluate;
use crate::game::frame::Frame;

pub struct PlacementNode {
    /// Frames that have confirmed_on_bottom as true.
    pub placement: Frame,
    pub after_lock: Frame,
    pub parent: Option<Rc<PlacementNode>>,
    pub score: OrderedFloat<f32>,
    pub level: usize,
}

impl PlacementNode {
    pub fn new(
        placement: &Frame,
        after_lock: &Frame,
        parent: Option<Rc<PlacementNode>>,
        evaluator: &impl Evaluate,
    ) -> Rc<Self> {
        Rc::new(PlacementNode {
            score: evaluator.eval(after_lock, false),
            parent: parent.clone(),
            placement: placement.clone(),
            after_lock: after_lock.clone(),
            level: parent.as_ref().map_or(0, |p| p.level + 1),
        })
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn get_path_from_root(&self) -> Vec<Frame> {
        let mut path = vec![self.placement.clone()];
        let mut current = self.parent.clone();
        while let Some(node) = current {
            path.push(node.placement.clone());
            current = node.parent.clone();
        }
        path.reverse();
        path
    }
}

impl Ord for PlacementNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}
impl PartialOrd for PlacementNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for PlacementNode {
    fn eq(&self, other: &Self) -> bool {
        self.level == other.level && self.placement == other.placement
    }
}
impl Eq for PlacementNode {}

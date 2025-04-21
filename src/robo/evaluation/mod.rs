pub mod default_eval;
// pub mod depra_eval;

use ordered_float::OrderedFloat;

use crate::tetris_core::engine::{BitBoard, BoardData};

pub trait Evaluate {
    fn eval(&self, board: &BitBoard, board_data: &BoardData, verbose: bool) -> OrderedFloat<f32>;
    // fn eval_verbose(&self, frame: &Frame) -> OrderedFloat<f32>;
}

pub struct NoEval {}
impl Evaluate for NoEval {
    fn eval(&self, _board: &BitBoard, _board_data: &BoardData, verbose: bool) -> OrderedFloat<f32> {
        if verbose {
            println!("noeval")
        }
        OrderedFloat(0.0)
    }

    // fn eval(&self, frame: &Frame) -> OrderedFloat<f32> {
    //     self.eval(frame)
    // }
}

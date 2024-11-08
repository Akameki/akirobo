//! Assigning a score to a game state.

use std::{cmp, vec};

use crate::botris::game_info::BOARD_HEIGHT;

use super::{frame::Frame, matrix::Matrix};

pub struct Evaluator {
    heights: [i32; 10],
    frame: Frame,
}

impl Evaluator {
    pub fn new(frame: Frame) -> Self {
        Evaluator {
            heights: [0; 10],
            frame,
        }
    }

    pub fn eval(&mut self, verbose: bool    ) -> f64 {
        self.pre_calculations();

        let mut total_score = 0.0;

        let eval_fns: Vec<(fn(&Evaluator) -> f64, f64, &str)> = vec![
            (Self::future_attack, 1.0, "attack"),
            (Self::max_height, 0.5, "max_height"),
            (Self::individual_holes, 0.3, "total_holes"),
            (Self::holes, 3.0, "hole_clusters"),
            (Self::bumpiness, 0.5, "bumpiness"),
            (Self::exposed_dependencies, 1.0, "exposed_dependencies"),
        ];

        for (eval_fn, weight, name) in eval_fns {
            let score = eval_fn(&self) * weight;
            total_score += score;
            if verbose {
                println!("{name}: {}", score);
            }
        } 
        if verbose {
            println!("Total score: {}", total_score);
        }
        total_score
    }

    fn pre_calculations(&mut self) {
        for x in 0..10 {
            self.heights[x] = 0;
            for y in (0..BOARD_HEIGHT).rev() {
                if self.frame.matrix[y][x] {
                    self.heights[x] = y as i32;
                    break;
                }
            }
        }
    }

    // Various evalutaions below

    fn future_attack(&self) -> f64 {
        self.frame.future_attack as f64
    }

    fn max_height(&self) -> f64 {
        0.0 - self.heights.iter().max().unwrap().clone() as f64
    }

    fn individual_holes(&self) -> f64 {
        let mut score = 0;
        for x in 0..10 {
            let mut holes = 0; // number of holes in the column so far
            for y in 0..BOARD_HEIGHT {
                if self.frame.matrix[y][x] {
                    score -= holes;
                } else {
                    holes += 1;
                }
            }
        }
        score as f64
    }

    fn holes(&self) -> f64 {
        let mut holes = 0;
        for x in 0..10 {
            let mut over_empty_space = 0;
            for y in 0..BOARD_HEIGHT {
                if self.frame.matrix[y][x] {
                    holes -= over_empty_space;
                    over_empty_space = 0;
                } else {
                    over_empty_space = 1;
                }
            }
        }
        holes as f64
    }

    fn bumpiness(&self) -> f64 {
        let mut bumpiness = 0;
        for i in 0..9 {
            let bump = (self.heights[i] - self.heights[i + 1]).abs();
            bumpiness -= bump;
        }
        bumpiness as f64
    }

    fn exposed_dependencies(&self) -> f64 {
        let mut score = 0;
        let mut wells = [0; 10]; // 10 columns + 2 for walls
        let mut highest_well = 0; // do not punish the highest well
        let heights = &self.heights;

        println!("{:?}", heights);

        // take minimum of height differences of left and right columns
        // punish if the difference is more than 1
        wells[0] = cmp::max(heights[1] - heights[0], 0);
        wells[9] = cmp::max(heights[8] - heights[9], 0);
        for i in 1..9 {

            let to_left = cmp::max(heights[i - 1] - heights[i], 0);
            let to_right = cmp::max(heights[i + 1] - heights[i], 0);

            let well = cmp::min(to_left, to_right);
            // highest_well = cmp::max(highest_well, well);
            wells[i] = well;

        }
        for well in wells.iter() {
            // highest_well = cmp::max(highest_well, *well);
            match well {
                0 => score -= 0,
                1 => score -= 0,
                2 => score -= 1,
                x => {
                    score -= x
                }
            }
        }

        println!("{:?}", wells);

        (score + highest_well) as f64
    }
}
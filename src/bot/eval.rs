//! Wraps a Frame with an evaluation

use std::{cmp, vec};

use crate::botris::game_info::BOARD_HEIGHT;

use crate::game::frame::Frame;

#[derive(Debug, Clone)]
pub struct Evaluator {
    pub frame: Frame,
    pub score: f64,
    
    heights: [i32; 10],
    verbose: bool,
}

impl Evaluator {
    pub fn new(frame: Frame) -> Self {
        Evaluator {
            frame,
            heights: [0; 10],
            score: f64::NAN,
            verbose: false,
        }
    }

    pub fn eval(&mut self, verbose: bool    ) -> f64 {
        self.pre_calculations();
        self.verbose = verbose;

        let mut total_score = 0.0;

        let eval_fns: Vec<(fn(&Evaluator) -> f64, f64, &str)> = vec![
            (Self::attacks, 2.0, "attack"),
            (Self::max_height, 1.0, "max_height"),
            (Self::individual_holes, 0.3, "total_holes"),
            (Self::holes, 3.0, "hole_clusters"),
            (Self::bumpiness, 0.5, "bumpiness"),
            (Self::exposed_dependencies, 1.0, "exposed_dependencies"),
        ];

        for (eval_fn, weight, name) in eval_fns {
            let score = eval_fn(self) * weight;
            total_score += score;
            if verbose {
                println!("{name}: {}", score);
            }
        } 
        if verbose {
            println!("Total score: {}", total_score);
        }
        self.score = total_score;
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

    fn attacks(&self) -> f64 {
        let mut score = 0.0;
        score += self.frame.future_attack as f64;
        score += if self.frame.b2b { 1 } else { 0 } as f64;
        score
    }

    fn max_height(&self) -> f64 {
        match self.heights.iter().max().unwrap() {
            0 => 0.0,
            1..=4 => -4.0,
            5|6 => -5.0,
            7|8 => -6.0,
            9|10 => -7.0,
            11 => -8.0,
            12 => -10.0,
            13 => -12.0,
            14 => -14.0,
            15 => -16.0,
            16 => -17.0,
            17 => -20.0,
            18 => -25.0,
            19 => -30.0,
            _ => -50.0,
        }
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

        if self.verbose { println!("{:?}", heights); }

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
        for well in wells {
            highest_well = cmp::max(highest_well, well);
            match well {
                0 => score -= 0,
                1 => score -= 0,
                2 => score -= 1,
                x => {
                    score -= x
                }
            }
        }

        if self.verbose { println!("{:?}", wells); }

        (score + highest_well) as f64
    }
}
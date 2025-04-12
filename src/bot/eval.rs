//! Wraps a Frame with an evaluation

use std::{cmp::{self, max}, vec};

use owo_colors::OwoColorize;

use crate::{botris::game_info::BOARD_HEIGHT, game::frame::Frame};

#[derive(Debug, Clone)]
pub struct Evaluator {
    pub frame: Frame,
    pub score: f64,

    heights: [usize; 10],
    verbose: bool,
}

impl Evaluator {
    pub fn new(frame: Frame, verbose: bool) -> Self {
        Evaluator { frame, heights: [0; 10], score: f64::NAN, verbose }
    }

    pub fn eval(&mut self) -> f64 {
        self.pre_calculations();

        let mut total_score = 0.0;

        let eval_fns: [(fn(&Self) -> f64, f64, &str); 4] = [
            (Self::attacks, 1.0, "attack"),
            (Self::max_height, 1.0, "max_height"),
            // (Self::blocks_over_empty_spaces, 2.0, "total_holes"),
            (Self::holes, 3.0, "hole_clusters"),
            (Self::bumpiness, 0.35, "bumpiness"),
            // (Self::dependencies, 1.0, "exposed_dependencies"),
        ];

        for (eval_fn, weight, name) in eval_fns {
            let score = eval_fn(self) * weight;
            total_score += score;
            if self.verbose {
                println!("{name}: {}", score);
            }
        }
        if self.verbose {
            println!("{} {}", "Total score:".bold(), total_score.bold());
        }
        self.score = total_score;
        total_score
    }

    fn pre_calculations(&mut self) {
        for x in 0..10 {
            self.heights[x] = 0;
            for y in (0..BOARD_HEIGHT).rev() {
                if self.frame.matrix[y][x] {
                    self.heights[x] = y;
                    break;
                }
            }
        }
    }

    // Various evalutaions below

    // attacks = good !
    fn attacks(&self) -> f64 {
        let mut score = 0.0;
        score += self.frame.future_attack as f64;
        score += if self.frame.b2b { 1 } else { 0 } as f64;
        score
    }

    // high board = bad !
    fn max_height(&self) -> f64 {
        match self.heights.iter().max().unwrap() {
            0 => 0.0,
            1 | 2 => -1.0,
            3 | 4 => -2.0,
            5 | 6 => -5.0,
            7 | 8 => -6.0,
            9 | 10 => -7.0,
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

    // blocks above spaces are bad ! especially if multiple spaces!
    fn blocks_over_empty_spaces(&self) -> f64 {
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

    // blocks over gaps are bad!
    fn holes(&self) -> f64 {
        let mut score = 0;
        for x in 0..10 {
            let mut hole_present = false;
            for y in 0..BOARD_HEIGHT {
                if self.frame.matrix[y][x] {
                    score -= hole_present as i32;
                    hole_present = false;
                } else {
                    hole_present = true;
                }
            }
        }
        score as f64
    }

    fn bumpiness(&self) -> f64 {
        let mut bumpiness = 0;
        for i in 0..9 {
            let bump = (self.heights[i] as i32 - self.heights[i + 1] as i32).abs();
            bumpiness -= bump;
        }
        bumpiness as f64
    }

    // tall wells are bad!
    fn dependencies(&self) -> f64 {
        let score = 0.0;
        // for x in 1..9 {
        //     let left = 0;
        //     let right = 0;
        //     let mut counting = false;
        //     for y in 0..max(self.heights[x-1], self.heights[x+1]) {
        //         if !self.frame.matrix[y][x] {
        //             if counting {

        //             } else {
        //                 counting = true;
        //             }
        //         }
        //     }
        // }
        score


        // let mut score = 0;
        // let mut wells = [0; 10]; // 10 columns + 2 for walls
        // let mut highest_well = 0; // do not punish the highest well
        // let heights = &self.heights;

        // if self.verbose {
        //     println!("{:?}", heights);
        // }

        // // take minimum of height differences of left and right columns
        // // punish if the difference is more than 1
        // wells[0] = cmp::max(heights[1] - heights[0], 0);
        // wells[9] = cmp::max(heights[8] - heights[9], 0);
        // for i in 1..9 {
        //     let to_left = cmp::max(heights[i - 1] - heights[i], 0);
        //     let to_right = cmp::max(heights[i + 1] - heights[i], 0);

        //     let well = cmp::min(to_left, to_right);
        //     // highest_well = cmp::max(highest_well, well);
        //     wells[i] = well;
        // }
        // for well in wells {
        //     // highest_well = cmp::max(highest_well, well); /// TODO disabled.
        //     match well {
        //         0 => score -= 0,
        //         1 => score -= 0,
        //         2 => score -= 1,
        //         x => score -= x,
        //     }
        // }

        // if self.verbose {
        //     println!("{:?}", wells);
        // }

        // (score + i32::min(highest_well, 4)) as f64
    }
}

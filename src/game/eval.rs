//! Assigning a score to a game state.

use std::{cmp, vec};

use crate::botris::game_info::BOARD_HEIGHT;

use super::{frame::Frame, matrix::Matrix};


pub fn eval(frame: &Frame) -> i32 {
    let mut score = 0;

    score += max_height(frame);
    score += holes(frame);
    score += exposed_dependencies(frame);
    score
}



fn max_height(frame: &Frame) -> i32 {
    let mut max = 0;
    for row in frame.matrix.iter().rev() {
        for cell in row.iter() {
            if *cell {
                return max;
            }
        }
        max += 1;
    }
    max
}

fn holes(frame: &Frame) -> i32 {
    let mut holes = 0;
    for x in 0..10 {
        let mut gap = 0;
        for y in 0..BOARD_HEIGHT {
            if frame.matrix[y][x] {
                if gap > 0 { // check if there was a gap underneath
                    holes += 1;
                }
                gap = 0;
            } else {
                gap += 1;
            }
        }
    }
    -holes
}

fn exposed_dependencies(frame: &Frame) -> i32 {
    let mut score = 0;
    let mut heights = [BOARD_HEIGHT as i32; 12]; // 10 columns + 2 for walls
    let mut highest_well = 0; // do not punish the highest well
    for x in 0..10 {
        for y in (0..BOARD_HEIGHT).rev() {
            if frame.matrix[y][x] {
                heights[x + 1] = y as i32;
                break;
            }
        }
    }

    // take minimum of height differences of left and right columns
    // punish if the difference is more than 1
    for i in 1..11 {

        let to_left = cmp::max(heights[i - 1] - heights[i], 0);
        let to_right = cmp::max(heights[i] - heights[i + 1], 0);

        let diff = cmp::min(to_left, to_right);

        match diff {
            0 => score += 1,
            1 => score += 0,
            2 => score -= 1,
            x => {
                score -= x + 2
            }
        }
    }

    score + highest_well
}
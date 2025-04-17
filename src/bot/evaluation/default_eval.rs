use std::{
    array,
    cmp::{max, min},
};

use ordered_float::OrderedFloat;
use owo_colors::OwoColorize;

use super::Evaluate;
use crate::{botris::game_info::BOARD_HEIGHT, game::frame::Frame};

struct DefaultEvalData<'a> {
    frame: &'a Frame,
    heights: [i32; 10],
    max_height: usize,
}
// impl Default for DefaultEvalData<'a> {
//     fn default() -> Self {
//         Self { frame: &, heights: heights: [0; 10], max_height: 999 }
//     }
// }

pub struct DefaultEval {}
impl Evaluate for DefaultEval {
    fn eval(&self, frame: &Frame, verbose: bool) -> OrderedFloat<f32> {
        let mut eval = 0.0;
        let mut data = DefaultEvalData { frame, heights: [0; 10], max_height: 99 };
        for (eval_fn, weight, name) in Self::SETTINGS {
            let score = eval_fn(&mut data);
            let weighted = score * weight;
            eval += weighted;
            if verbose {
                println!("{name:>10}: {weighted:>5.1} = {score:>5.1}*{weight:>3.1}");
            }
        }
        if verbose {
            println!("{:>10}: {:>5.1}", "Total".bold(), eval.bold());
        }
        OrderedFloat(eval)
    }
}

impl DefaultEval {
    const SETTINGS: [(fn(&mut DefaultEvalData) -> f32, f32, &str); 8] = [
        (Self::bumpy, 0.3, "bumpy"),
        (Self::b2b, 3.0, "b2b"),
        (Self::attack, 0.3, "attack"),
        (Self::height, 1.0, "height"),
        (Self::avg_height, 0.0, "a_height"),
        (Self::holes, 3.0, "holes"),
        (Self::garbage, 2.0, "garbage"),
        (Self::depends, 1.0, "depends"),
    ];

    // includes covered bumpiness !!!
    fn bumpy(data: &mut DefaultEvalData) -> f32 {
        let frame = data.frame;
        let mut score = 0;

        let mut counting_left: [bool; 10] = array::from_fn(|x| !frame.matrix[0][x]);
        let mut counting_right: [bool; 10] = array::from_fn(|x| !frame.matrix[0][x]);
        counting_left[0] = false;
        counting_right[9] = false;

        for y in 0..BOARD_HEIGHT {
            for x in 0..=9 {
                if x > 0 {
                    if y > 0 && frame.matrix[y - 1][x - 1] && !frame.matrix[y][x - 1] {
                        counting_right[x - 1] = true;
                    }
                    if counting_right[x - 1] {
                        if frame.matrix[y][x - 1] || !frame.matrix[y][x] {
                            counting_right[x - 1] = false;
                        } else {
                            score -= 1;
                        }
                    }
                }
                if x < 9 {
                    if y > 0 && frame.matrix[y - 1][x + 1] && !frame.matrix[y][x + 1] {
                        counting_left[x + 1] = true;
                    }
                    if counting_left[x + 1] {
                        if frame.matrix[y][x + 1] || !frame.matrix[y][x] {
                            counting_left[x + 1] = false;
                        } else {
                            score -= 1;
                        }
                    }
                }

                if frame.matrix[y][x] {
                    data.heights[x] = y as i32;
                }
            }
        }

        data.max_height = *data.heights.iter().max().unwrap() as usize;

        score as f32
    }

    fn b2b(data: &mut DefaultEvalData) -> f32 {
        let frame = data.frame;
        if frame.b2b {
            1.0
        } else {
            0.0
        }
    }
    fn attack(data: &mut DefaultEvalData) -> f32 {
        let frame = data.frame;
        let mut score = 0.0;
        score += frame.stored_attack as f32;
        score
    }

    // high board = bad !
    fn height(data: &mut DefaultEvalData) -> f32 {
        match data.max_height {
            0 => 0.0,
            1 => 0.0,
            2 => 0.0,
            3 => 0.0,
            4 => 0.0,
            5 | 6 => 0.0,
            7 | 8 => -1.5,
            9 | 10 => -3.0,
            11 => -5.0,
            12 => -7.0,
            13 => -10.5,
            14 => -14.0,
            15 => -18.0,
            16 => -22.0,
            17 => -26.0,
            18 => -50.0,
            19 => -70.0,
            _ => -150.0,
        }
    }

    fn avg_height(data: &mut DefaultEvalData) -> f32 {
        // let avg_height = data.heights.iter().sum::<i32>() as f32 / 10.0;
        // match avg_height {
        //     h if h <= 4.0 => h - 4.0,
        //     h if h <= 6.0 => 4.0 - h,
        //     h => (4.0 - h) * 1.5,
        // }
        0.0
    }
    // block over (contiguous) space = bad!
    // TODO: calculate at same time as other evals
    fn holes(data: &mut DefaultEvalData) -> f32 {
        let frame = data.frame;
        let mut score = 0.0;
        for x in 0..10 {
            let mut hole_present = 0.0;
            for y in 0..frame.matrix.len() {
                if frame.matrix[y][x] {
                    if hole_present == 0.0 {
                        continue;
                    }
                    score -= hole_present;
                    hole_present = match hole_present {
                        1.0 => 0.2,
                        0.2 => 0.1,
                        0.1 => 0.05,
                        0.05 => 0.0,
                        _ => unreachable!(),
                    }
                } else {
                    hole_present = 1.0;
                }
            }
        }
        score
    }

    // garbage = bad!
    fn garbage(data: &mut DefaultEvalData) -> f32 {
        -(data.frame.simulated_garbage as f32)
    }

    // stacks with 1 wide wells are bad! except maybe 9-0 :p
    fn depends(data: &mut DefaultEvalData) -> f32 {
        let mut score = 0.0;
        for x in 1..=8 {
            score += match max(min(data.heights[x - 1], data.heights[x + 1]) - data.heights[x], 0) {
                0 | 1 => 0.0,
                2 => -1.0,
                3 | 4 => -2.0,
                n => -(n as f32 - 2.5),
            }
        }
        let col1 = max(data.heights[1] - data.heights[0], 0);
        let col10 = max(data.heights[8] - data.heights[9], 0);
        for h in [col1, col10] {
            score += match h {
                0 => 0.0,
                1 => -0.5,
                2 => -1.5,
                3 | 4 => -2.0,
                n => -(n as f32 - 2.5),
            }
        }

        score
    }

    // col 1
    // let mut counting = !frame.matrix[0][0];
    // for y in 0..BOARD_HEIGHT {
    //     // if we are at an empty space above a block, we add the heights of walls next to us.
    //     if y > 0 && frame.matrix[y-1][0] && !frame.matrix[y][0] {
    //         counting = true;
    //     }
    //     // we stop counting if we hit a block (impossible when same iter as start counting)
    //     // or stop counting if we made it over a neighbor's wall
    //     if counting {
    //         if frame.matrix[y][0] || !frame.matrix[y][1] {
    //             counting = false;
    //         } else {
    //             score += 1;
    //         }
    //     }
    // }
}

#[cfg(test)]
mod test {
    use super::DefaultEval;
    use crate::{
        bot::evaluation::{default_eval::DefaultEvalData, Evaluate},
        game::frame::Frame,
    };

    #[test]
    fn test_evals() {
        // 40
        // let frame0 = Frame::from_strings(vec![
        //     "    []  []  []  [][]".to_string(),
        //     "  []  []  []    []  ".to_string(),
        //     "[]  []  []  []  []  ".to_string(),
        //     "  [][]  []  []  []  ".to_string(),
        //     "  []  []  []  [][][]".to_string(),
        //     "[][]    [][][][][]  ".to_string(),
        // ]);
        let frame = Frame::from_strings(&[
            "[][]  [][][][][][][]",
            "[]        [][][][][]",
            "[][]  [][][][]  [][]",
            "[][][][][][][]  [][]",
            "  [][][][][]    [][]",
            "[][][][][][][]  [][]",
        ]);
        println!("{}", frame);
        assert_eq!(
            DefaultEval::bumpy(&mut DefaultEvalData {
                frame: &frame,
                heights: [0; 10],
                max_height: 99
            }),
            11.0
        )
    }

    #[test]
    fn compare_evals() {
        // should "manually" clear lines by commenting.
        let frame1 = Frame::from_strings(&[
            "                    ",
            "                    ",
            "        ██          ",
            "        ██          ",
            "      ████          ",
            "      [][]          ",
            "        [][]        ",
        ]);
        let frame2 = Frame::from_strings(&[
            "                    ",
            "      [][]  ██      ",
            "        [][]██████  ",
        ]);
        let eval = DefaultEval {};
        println!("{}", frame1);
        eval.eval(&frame1, true);
        println!("{}", frame2);
        eval.eval(&frame2, true);
    }
}

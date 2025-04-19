use std::{
    array,
    cmp::{max, min},
};

use ordered_float::OrderedFloat;
use owo_colors::OwoColorize;

use super::Evaluate;
use crate::{
    botris::game_info::BOARD_HEIGHT,
    tetris_core::engine::{Board, BoardData},
};

struct DefaultEvalData<'a> {
    board: &'a Board,
    board_data: &'a BoardData,
    heights: [i32; 10],
    stack_height: usize,
}
// impl Default for DefaultEvalData<'a> {
//     fn default() -> Self {
//         Self { board: &, heights: heights: [0; 10], max_height: 999 }
//     }
// }

pub struct DefaultEval {}
impl Evaluate for DefaultEval {
    fn eval(&self, board: &Board, board_data: &BoardData, verbose: bool) -> OrderedFloat<f32> {
        let mut eval = 0.0;
        let mut data = DefaultEvalData { board, board_data, heights: [0; 10], stack_height: 99 };
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
        (Self::combob2b, 0.5, "combob2b"),
        (Self::attack, 1.0, "attack"),
        (Self::height, 1.0, "height"),
        (Self::avg_height, 0.0, "a_height"),
        (Self::holes, 3.0, "holes"),
        (Self::garbage, 2.0, "garbage"),
        (Self::depends, 1.0, "depends"),
    ];

    // includes covered bumpiness !!!
    fn bumpy(DefaultEvalData { board, heights, stack_height, .. }: &mut DefaultEvalData) -> f32 {
        let mut score = 0;

        let mut counting_left: [bool; 10] = array::from_fn(|x| !board[0][x]);
        let mut counting_right: [bool; 10] = array::from_fn(|x| !board[0][x]);
        counting_left[0] = false;
        counting_right[9] = false;

        for y in 0..BOARD_HEIGHT {
            for x in 0..=9 {
                if x > 0 {
                    if y > 0 && board[y - 1][x - 1] && !board[y][x - 1] {
                        counting_right[x - 1] = true;
                    }
                    if counting_right[x - 1] {
                        if board[y][x - 1] || !board[y][x] {
                            counting_right[x - 1] = false;
                        } else {
                            score -= 1;
                        }
                    }
                }
                if x < 9 {
                    if y > 0 && board[y - 1][x + 1] && !board[y][x + 1] {
                        counting_left[x + 1] = true;
                    }
                    if counting_left[x + 1] {
                        if board[y][x + 1] || !board[y][x] {
                            counting_left[x + 1] = false;
                        } else {
                            score -= 1;
                        }
                    }
                }

                if board[y][x] {
                    heights[x] = y as i32;
                }
            }
        }

        *stack_height = *heights.iter().max().unwrap() as usize;

        score as f32
    }

    fn combob2b(DefaultEvalData { board_data, .. }: &mut DefaultEvalData) -> f32 {
        let mut score = 0;
        // score += board_data.combo;
        if board_data.b2b {
            score += 1;
        }
        score as f32
    }
    fn attack(DefaultEvalData { board_data, .. }: &mut DefaultEvalData) -> f32 {
        let mut score = 0.0;
        score += board_data.cummulative_attack as f32;
        score
    }

    // high board = bad !
    fn height(DefaultEvalData { stack_height, .. }: &mut DefaultEvalData) -> f32 {
        match stack_height {
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

    fn avg_height(DefaultEvalData { heights, .. }: &mut DefaultEvalData) -> f32 {
        // let avg_height = heights.iter().sum::<i32>() as f32 / 10.0;
        // match avg_height {
        //     h if h <= 4.0 => h - 4.0,
        //     h if h <= 6.0 => 4.0 - h,
        //     h => (4.0 - h) * 1.5,
        // }
        0.0
    }
    // block over (contiguous) space = bad!
    // TODO: calculate at same time as other evals
    fn holes(DefaultEvalData { board, .. }: &mut DefaultEvalData) -> f32 {
        let mut score = 0.0;
        for x in 0..10 {
            let mut hole_present = 0.0;
            for y in 0..board.len() {
                if board[y][x] {
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
    fn garbage(DefaultEvalData { board_data, .. }: &mut DefaultEvalData) -> f32 {
        -(board_data.simulated_garbage as f32)

    }

    // stacks with 1 wide wells are bad! except maybe 9-0 :p
    fn depends(DefaultEvalData { heights, .. }: &mut DefaultEvalData) -> f32 {
        let mut score = 0.0;
        for x in 1..=8 {
            score += match max(min(heights[x - 1], heights[x + 1]) - heights[x], 0) {
                0 | 1 => 0.0,
                2 => -1.0,
                3 | 4 => -2.0,
                n => -(n as f32 - 2.5),
            }
        }
        let col1 = max(heights[1] - heights[0], 0);
        let col10 = max(heights[8] - heights[9], 0);
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
}

#[cfg(test)]
mod test {
    use super::DefaultEval;
    use crate::{
        evaluation::{default_eval::DefaultEvalData, Evaluate},
        tetris_core::engine::{print_board, strs_to_board, BoardData},
    };

    #[test]
    fn test_evals() {
        // 40
        // let board0 = board::from_strings(vec![
        //     "    []  []  []  [][]".to_string(),
        //     "  []  []  []    []  ".to_string(),
        //     "[]  []  []  []  []  ".to_string(),
        //     "  [][]  []  []  []  ".to_string(),
        //     "  []  []  []  [][][]".to_string(),
        //     "[][]    [][][][][]  ".to_string(),
        // ]);
        let board = strs_to_board(&[
            "[][]  [][][][][][][]",
            "[]        [][][][][]",
            "[][]  [][][][]  [][]",
            "[][][][][][][]  [][]",
            "  [][][][][]    [][]",
            "[][][][][][][]  [][]",
        ]);
        // print_board(&board);
        assert_eq!(
            DefaultEval::bumpy(&mut DefaultEvalData {
                board: &board,
                board_data: &BoardData::default(),
                heights: [0; 10],
                stack_height: 99,
            }),
            -11.0
        )
    }

    #[test]
    fn compare_evals() {
        // don't forget to "manually" clear lines (by simpling commenting)
        let board1 = strs_to_board(&[
            "                    ",
            "                    ",
            "        ██          ",
            "        ██          ",
            "      ████          ",
            "      [][]          ",
            "        [][]        ",
        ]);
        let board2 = strs_to_board(&[
            "                    ",
            "      [][]  ██      ",
            "        [][]██████  ",
        ]);
        let board_data = BoardData::default();
        let eval = DefaultEval {};
        print_board(&board1);
        eval.eval(&board1, &board_data, true);
        print_board(&board2);
        eval.eval(&board2, &board_data, true);
    }
}

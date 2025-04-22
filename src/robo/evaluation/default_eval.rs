use std::cmp::{max, min};

use ordered_float::OrderedFloat;
use owo_colors::OwoColorize;

use super::Evaluate;
use crate::tetris_core::engine::{BitBoard, BoardData, BITBOARD_HEIGHT};

struct DefaultEvalData<'a> {
    board: &'a BitBoard,
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
    fn eval(&self, board: &BitBoard, board_data: &BoardData, verbose: bool) -> OrderedFloat<f32> {
        #[allow(clippy::type_complexity)]
        let heuristics: [(fn(&DefaultEvalData) -> f32, f32, &str); 6] = [
            (Self::bumpy, 0.2, "bumpy"),
            // (Self::_combob2b, 0.5, "combob2b"),
            (Self::attack, 1.0, "attack"),
            (Self::height, 1.0, "height"),
            // (Self::_avg_height, 0.0, "a_height"),
            (Self::holes, 2.0, "holes"),
            (Self::garbage, 1.0, "garbage"),
            (Self::depends, 1.0, "depends"),
        ];
        let mut eval = 0.0;
        let data = DefaultEvalData {
            board,
            board_data,
            heights: std::array::from_fn(|col| board.column_height(col) as i32),
            stack_height: board.stack_height(),
        };
        for (eval_fn, weight, name) in heuristics {
            let score = eval_fn(&data);
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
    // diff from XORing columns
    fn bumpy(DefaultEvalData { board, .. }: &DefaultEvalData) -> f32 {
        // let mut score = 0;
        let mut bumps = 0;

        // let mut counting_left: [bool; 10] = array::from_fn(|x| !board.at(0, x));
        // let mut counting_right: [bool; 10] = array::from_fn(|x| !board.at(0, x));
        // counting_left[0] = false;
        // counting_right[9] = false;

        // for y in 0..BOARD_HEIGHT {
        //     for x in 0..=9 {
        //         if x > 0 {
        //             if y > 0 && board.at(y - 1, x - 1) && !board.at(y, x - 1) {
        //                 counting_right[x - 1] = true;
        //             }
        //             if counting_right[x - 1] {
        //                 if board.at(y, x - 1) || !board.at(y, x) {
        //                     counting_right[x - 1] = false;
        //                 } else {
        //                     score -= 1;
        //                 }
        //             }
        //         }
        //         if x < 9 {
        //             if y > 0 && board.at(y - 1, x + 1) && !board.at(y, x + 1) {
        //                 counting_left[x + 1] = true;
        //             }
        //             if counting_left[x + 1] {
        //                 if board.at(y, x + 1) || !board.at(y, x) {
        //                     counting_left[x + 1] = false;
        //                 } else {
        //                     score -= 1;
        //                 }
        //             }
        //         }
        //     }
        // }

        // xor from column to column and sum up number of 1s
        let mut prev_col = board.cols[0];
        for x in 1..=9 {
            let col = board.cols[x];
            bumps += (prev_col ^ col).count_ones();
            prev_col = col;
        }

        -(bumps as f32)
    }

    fn _combob2b(DefaultEvalData { board_data, .. }: &DefaultEvalData) -> f32 {
        let mut score = 0;
        // score += board_data.combo;
        if board_data.b2b {
            score += 1;
        }
        score as f32
    }
    fn attack(DefaultEvalData { board_data, .. }: &DefaultEvalData) -> f32 {
        let mut score = 0.0;
        score += board_data.cummulative_attack as f32;
        score
    }

    // high board = bad !
    fn height(DefaultEvalData { stack_height, .. }: &DefaultEvalData) -> f32 {
        match stack_height {
            0 => 0.0,
            1 => 0.0,
            2 => 0.0,
            3 => 0.0,
            4 => 0.0,
            5 => -1.0,
            6 => -2.0,
            7 => -3.0,
            8 => -4.0,
            9 => -5.5,
            10 => -7.0,
            11 => -9.0,
            12 => -11.0,
            13 => -14.0,
            14 => -18.0,
            15 => -22.0,
            16 => -26.0,
            17 => -30.0,
            18 => -35.0,
            19 => -50.0,
            _ => -70.0,
        }
    }

    fn _avg_height(DefaultEvalData { heights: _, .. }: &DefaultEvalData) -> f32 {
        // let avg_height = heights.iter().sum::<i32>() as f32 / 10.0;
        // match avg_height {
        //     h if h <= 4.0 => h - 4.0,
        //     h if h <= 6.0 => 4.0 - h,
        //     h => (4.0 - h) * 1.5,
        // }
        0.0
    }
    // block over (contiguous) space = bad!
    fn holes(DefaultEvalData { board, .. }: &DefaultEvalData) -> f32 {
        let mut score = 0.0;
        for mut col in board.cols {
            if col == u32::MAX {
                continue;
            }
            col >>= col.trailing_ones();
            while col.trailing_zeros() as usize != BITBOARD_HEIGHT {
                col >>= col.trailing_zeros();
                score += match col.trailing_ones() {
                    0 => panic!(),
                    1 => -1.0,
                    2 => -1.25,
                    3 => -1.5,
                    4 => -1.75,
                    _ => -2.0,
                };
                col >>= col.trailing_ones();
            }
        }
        score
    }

    // garbage = bad!
    fn garbage(DefaultEvalData { board_data, .. }: &DefaultEvalData) -> f32 {
        -(board_data.simulated_garbage as f32)
    }

    // stacks with 1 wide wells are bad! except maybe 9-0 :p
    fn depends(DefaultEvalData { heights, .. }: &DefaultEvalData) -> f32 {
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
        evaluation::Evaluate,
        tetris_core::engine::{BitBoard, BoardData},
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
        let board = BitBoard::from_strs(&[
            "[][]  [][][][][][][]",
            "[]        [][][][][]",
            "[][]  [][][][]  [][]",
            "[][][][][][][]  [][]",
            "  [][][][][]    [][]",
            "[][][][][][][]  [][]",
        ]);
        board.print_board(None);
        DefaultEval {}.eval(&board, &Default::default(), true);
        // assert_eq!(
        //     DefaultEval::bumpy(&Default::default()),
        //     -11.0
        // )
    }

    #[test]
    fn compare_evals() {
        // don't forget to "manually" clear lines (by simpling commenting)
        let board1 = BitBoard::from_strs(&[
            "                    ",
            "                    ",
            "        ██          ",
            "        ██          ",
            "      ████          ",
            "      [][]          ",
            "        [][]        ",
        ]);
        let board2 = BitBoard::from_strs(&[
            "                    ",
            "      [][]  ██      ",
            "        [][]██████  ",
        ]);
        let board_data = BoardData::default();
        let eval = DefaultEval {};
        board1.print_board(None);
        eval.eval(&board1, &board_data, true);
        board2.print_board(None);
        eval.eval(&board2, &board_data, true);
    }
}

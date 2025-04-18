use criterion::{black_box, criterion_group, criterion_main, Criterion};
use robo::{
    akirobo::Akirobo,
    botris::types::Piece,
    tetris_core::{frame::Frame, piece::FallingPiece},
};

// use std::time::Instant;

fn benchmark_movegen(c: &mut Criterion) {
    let pieces = [Piece::I, Piece::O, Piece::T, Piece::L, Piece::J, Piece::S, Piece::Z];

    // Define test boards
    let boards = [
        (
            "BOARD TSPIN",
            // 0b11111111,
            // 0b00111111,
            // 0b00011111,
            // 0b00001101,
            // 0b00000000,
            // 0b00000001,
            // 0b00000111,
            // 0b00011111,
            // 0b00111111,
            // 0b00111111,
            Frame::from_strings(&[
                "                  []",
                "                  []",
                "[][]            [][]",
                "[][][]        [][][]",
                "[][][]      [][][][]",
                "[][][][]    [][][][]",
                "[][][][]      [][][]",
                "[][][][][]  [][][][]",
            ]),
        ),
        (
            "BOARD DT CANNON",
            // 0b011111111,
            // 0b011111111,
            // 0b011110111,
            // 0b010000001,
            // 0b000100110,
            // 0b000111111,
            // 0b011111111,
            // 0b011111111,
            // 0b111111111,
            // 0b111111111,
            Frame::from_strings(&[
                "[][]                ",
                "[][][][]    [][][][]",
                "[][][][]      [][][]",
                "[][][][][][]  [][][]",
                "[][][][][]    [][][]",
                "[][][][][]      [][]",
                "[][][][][][]  [][][]",
                "[][][][][][]  [][][]",
                "[][][][][]  [][][][]",
            ]),
        ),
        (
            "BOARD TERRIBLE",
            // 0b111111111100,
            // 0b110000001100,
            // 0b110000001100,
            // 0b110011001100,
            // 0b110011001100,
            // 0b110011001100,
            // 0b110011001100,
            // 0b110011001100,
            // 0b000011000000,
            // 0b000011111111,
            Frame::from_strings(&[
                "    [][][][][][][][]",
                "    [][][][][][][][]",
                "                  []",
                "                  []",
                "[][][][][][][]    []",
                "[][][][][][][]    []",
                "[]                []",
                "[]                []",
                "[]  [][][][][][][][]",
                "[]  [][][][][][][][]",
                "[]                  ",
                "[]                  ",
            ]),
        ),
    ];

    let mut akirobo = Akirobo::new();

    for (name, board) in boards {
        let mut group = c.benchmark_group(format!("movegen: {}", name));

        for piece in pieces {
            let frame_with_piece = Frame {
                falling_piece: FallingPiece::new(piece),
                held: piece,
                can_hold: false,
                queue: vec![piece],
                ..board.clone()
            };
            group.bench_function(format!("{piece:?}"), |b| {
                b.iter(|| {
                    let moves = akirobo.move_gen(black_box(&frame_with_piece));
                    black_box(moves);
                })
            });
            println!("Generated: {}", akirobo.move_gen(&frame_with_piece).len());

            // Additional statistics can be collected using Criterion's measurement settings
            // which are configured in the criterion_group
        }

        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(1000)  // Increase for more stable results
        .measurement_time(std::time::Duration::from_secs(2))  // Longer measurement time
        .warm_up_time(std::time::Duration::from_millis(500));  // Warm-up period
    targets = benchmark_movegen
}

criterion_main!(benches);

//     auto func = [&] (Board b) {
//         const int count = 1000000;

//         // For each piece
//         for (i8 t = 0; t < 7; ++t) {
//             int64_t time = 0;
//             int64_t c = 0;

//             std::vector<int64_t> lists;
//             lists.reserve(1000000);

//             for (int i = 0; i < count; ++i) {
//                 auto time_start = chrono::high_resolution_clock::now();
//                 auto m = Shaktris::MoveGen::Smeared::god_movegen(b, queue[t]);
//                 auto time_stop = chrono::high_resolution_clock::now();

//                 auto dt = chrono::duration_cast<chrono::nanoseconds>(time_stop - time_start).count();

//                 c += m.size();
//                 time += dt;
//                 lists.push_back(dt);
//             }

//             // Calculate mean time & movegen count
//             time = time / count;
//             c = c / count;

//             // Calculate stdev
//             uint64_t sd = 0;
//             uint64_t max = 0;
//             uint64_t min = UINT64_MAX;

//             for (auto dt : lists) {
//                 sd += (dt - time) * (dt - time);
//                 max = std::max(max, uint64_t(dt));
//                 min = std::min(min, uint64_t(dt));
//             }

//             sd = sd / count;

//             cout << "    piece: " << to_char(queue[t]) << "    time: " << time << " ns" << "    stdev: " << std::sqrt(sd) << "    min: " << min << " ns"  << "    max: " << max << " ns" << "    count: " << c << endl;
//         }
//     };

//     return 0;
// }

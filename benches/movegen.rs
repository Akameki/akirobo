use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use pprof::criterion::{Output, PProfProfiler};
use robo::{botris::types::Piece, movegen::move_gen, tetris_core::engine::BitBoard};

// use std::time::Instant;

fn benchmark_movegen(c: &mut Criterion) {
    let pieces = [Piece::I, Piece::O, Piece::T, Piece::L, Piece::J, Piece::S, Piece::Z];

    // Define test boards
    let boards = [
        (
            "BOARD TSPIN",
            BitBoard::from_strs(&[
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
        // (
        //     "BOARD DT CANNON",
        //     BitBoard::from_strs(&[
        //         "[][]                ",
        //         "[][][][]    [][][][]",
        //         "[][][][]      [][][]",
        //         "[][][][][][]  [][][]",
        //         "[][][][][]    [][][]",
        //         "[][][][][]      [][]",
        //         "[][][][][][]  [][][]",
        //         "[][][][][][]  [][][]",
        //         "[][][][][]  [][][][]",
        //     ]),
        // ),
        // (
        //     "BOARD TERRIBLE",
        //     BitBoard::from_strs(&[
        //         "    [][][][][][][][]",
        //         "    [][][][][][][][]",
        //         "                  []",
        //         "                  []",
        //         "[][][][][][][]    []",
        //         "[][][][][][][]    []",
        //         "[]                []",
        //         "[]                []",
        //         "[]  [][][][][][][][]",
        //         "[]  [][][][][][][][]",
        //         "[]                  ",
        //         "[]                  ",
        //     ]),
        // ),
    ];

    // let mut akirobo = Akirobo::new();

    for (name, board) in boards {
        let mut group = c.benchmark_group(format!("movegen: {}", name));

        for piece in pieces {
            group.bench_function(format!("{piece:?}"), |b| {
                b.iter(|| black_box(move_gen(black_box(&board), black_box(piece))))
            });
        }

        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(500)
        .measurement_time(std::time::Duration::from_secs(2))
        .warm_up_time(std::time::Duration::from_millis(300));
        // .with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = benchmark_movegen
}

criterion_main!(benches);

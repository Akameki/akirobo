use criterion::{black_box, criterion_group, criterion_main, Criterion};
use robo::{
    botris::types::Piece,
    movegen::{move_gen, move_gen_with_action},
    tetris_core::engine::BitBoard,
};

fn bench_fibs(c: &mut Criterion) {
    let board = BitBoard::from_strs(&[
        "[][]                ",
        "[][][][]    [][][][]",
        "[][][][]      [][][]",
        "[][][][][][]  [][][]",
        "[][][][][]    [][][]",
        "[][][][][]      [][]",
        "[][][][][][]  [][][]",
        "[][][][][][]  [][][]",
        "[][][][][]  [][][][]",
    ]);
    let piece = Piece::T;
    let mut group = c.benchmark_group("movegens");
    group.bench_function("record_action", |b| {
        b.iter(|| black_box(move_gen_with_action(black_box(&board), black_box(piece))))
    });
    group.bench_function("no_action", |b| {
        b.iter(|| black_box(move_gen(black_box(&board), black_box(piece))))
    });
}

criterion_group!(benches, bench_fibs);
criterion_main!(benches);

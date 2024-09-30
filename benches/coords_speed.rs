use criterion::{black_box, criterion_group, criterion_main, Criterion};
use app::coords::chunk_coord::{ChunkCoord, MyHasher};
use std::hash::{DefaultHasher, Hash};
use rustc_hash::FxHasher;
use fnv::FnvHasher;

fn chunk_coord_default_hash(c: &mut Criterion) {
    let cc = ChunkCoord::new(1, 2);
    let mut hasher = DefaultHasher::new();
    c.bench_function("chunk_coord_default_hash", |b| b.iter(|| cc.hash(&mut hasher)));
}

fn chunk_coord_fx_rustc_hash(c: &mut Criterion) {
    let cc = ChunkCoord::new(1, 2);
    let mut hasher = FxHasher::default();
    c.bench_function("chunk_coord_fx_rustc_hash", |b| b.iter(|| cc.hash(&mut hasher)));
}

fn chunk_coord_fnv_hash(c: &mut Criterion) {
    let cc = ChunkCoord::new(1, 2);
    let mut hasher = FnvHasher::default();
    c.bench_function("chunk_coord_fnv_hash", |b| b.iter(|| cc.hash(&mut hasher)));
}

fn chunk_coord_my_hash(c: &mut Criterion) {
    let cc = ChunkCoord::new(1, 2);
    let mut hasher = MyHasher::new();
    c.bench_function("chunk_coord_my_hash", |b| b.iter(|| cc.hash(&mut hasher)));
}

criterion_group!(benches, chunk_coord_default_hash, chunk_coord_fx_rustc_hash, chunk_coord_fnv_hash, chunk_coord_my_hash);
criterion_main!(benches);
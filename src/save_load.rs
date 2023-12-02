use std::{path::Path, fs::File};
use std::io::prelude::*;

pub fn save_chunk(data: &[u8]) {
    let p = Path::new("./data/worlds/debug/chunk.bin");
    let mut f = File::create(p).unwrap();
    f.write_all(data).unwrap();
}

pub fn load_chunk() -> Vec<u8> {
    let mut buf = vec![];
    let p = Path::new("./data/worlds/debug/chunk.bin");
    let mut f = File::open(p).unwrap();
    f.read_to_end(&mut buf).unwrap();

    buf
}
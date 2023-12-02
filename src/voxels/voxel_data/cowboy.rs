use std::time::Instant;

use crate::bytes::ConstByteInterpretation;

#[derive(Debug)]
pub struct Cowboy {
    time: Instant,
}

impl Cowboy {
    pub fn new() -> Self {Self { time: Instant::now() }}

    pub fn animation_progress(&self) -> f32 {
        self.time.elapsed().as_secs_f32() % 1.0
    }
}

impl Default for Cowboy {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstByteInterpretation for Cowboy {
    fn to_bytes(&self) -> Box<[u8]> {
        Box::new([])
    }

    fn from_bytes(_: &[u8]) -> Self {
        Self::default()
    }

    fn size(&self) -> u32 {0}
}
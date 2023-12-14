use std::time::Instant;

use crate::bytes::BytesCoder;

#[derive(Debug)]
pub struct Cowboy {
    time: Instant,
}

impl Cowboy {
    #[inline] pub fn new() -> Self {Self { time: Instant::now() }}

    #[inline]
    pub fn animation_progress(&self) -> f32 {
        self.time.elapsed().as_secs_f32() % 1.0
    }
}

impl Default for Cowboy {
    #[inline] fn default() -> Self {Self::new()}
}

impl BytesCoder for Cowboy {
    fn encode_bytes(&self) -> Box<[u8]> { Box::new([]) }
    fn decode_bytes(_: &[u8]) -> Self { Self::default() }
}
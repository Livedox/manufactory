use std::time::Instant;

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
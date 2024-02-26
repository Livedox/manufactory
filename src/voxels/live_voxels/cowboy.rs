use std::time::Instant;



use crate::{direction::Direction};

use super::{LiveVoxelBehavior, LiveVoxelCreation};

#[derive(Debug)]
pub struct Cowboy {
    time: Instant,
}

impl Default for Cowboy {
    #[inline] fn default() -> Self {Self {
        time: Instant::now()
    }}
}

impl LiveVoxelBehavior for Cowboy {
    fn animation_progress(&self) -> f32 {
        self.time.elapsed().as_secs_f32() % 1.0
    }

    fn to_bytes(&self) -> Vec<u8> {
        vec![]
    }
}

impl LiveVoxelCreation for Cowboy {
    fn create(_: &Direction) -> Box<dyn LiveVoxelBehavior> {
        Box::new(Cowboy::default())
    }

    fn from_bytes(_: &[u8]) -> Box<dyn LiveVoxelBehavior> {
        Box::new(Cowboy::default())
    }
}
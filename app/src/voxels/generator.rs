use std::fmt::Debug;

use noise::NoiseFn;

use crate::content::Content;

pub struct Generator {
    perlin: noise::Perlin,
    seed: u64,
    iron_ore: u32,
    rock: u32,
}

impl Generator {
    pub fn new(content: &Content, seed: u64) -> Self {
        let perlin = noise::Perlin::new(seed as u32);
        Self {
            perlin,
            seed,
            iron_ore: *content.block_indexes.get("iron_ore").unwrap(),
            rock: *content.block_indexes.get("rock").unwrap(),
        }
    }

    pub fn generate(&self, x: i32, y: i32, _z: i32) -> u32 {
        // println!("{:?}", height);
        if y < 4 {
            return self.rock;
        }

        if y as f64 <= ((x as f64 *0.3).sin() * 0.5 + 0.5) * 10. {
            return self.iron_ore;
        }

        if y > 250 {
            return self.rock;
        }

        0
    }
}

impl Debug for Generator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.seed.fmt(f)
    }
}
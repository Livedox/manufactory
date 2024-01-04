use std::path::Path;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldData {
    pub name: String,
    pub creation_time: u64,
    pub seed: u64,
}

impl WorldData {
    pub fn new(name: String, creation_time: u64, seed: u64) -> Self {Self {
        name,
        creation_time,
        seed
    }}
}

impl Default for WorldData {
    fn default() -> Self {
        Self { name: String::from("Error"), creation_time: 0, seed: 0 }
    }
}

pub struct WorldLoader {
    pub worlds: Vec<WorldData>,
}

impl WorldLoader {
    pub fn new(path: &Path) -> Self {
        let mut worlds: Vec<WorldData> = vec![];

        for dir in std::fs::read_dir(path).unwrap() {
            let Ok(dir) = dir else {continue};
            let mut path = dir.path().clone();
            path.push("world.json");

            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(world) = serde_json::from_slice(&bytes) {
                    worlds.push(world);
                }
            }
        }

        Self {worlds}
    }
}
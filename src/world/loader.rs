use std::{path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}, fs};

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
    pub path_buf: PathBuf,
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

        Self {
            worlds,
            path_buf: path.to_path_buf()
        }
    }


    pub fn create_world(&mut self, name: &str, seed: u64) -> Result<(), ()> {
        let path = self.path_buf.join(name);
        println!("{:?}", path);
        if fs::read_dir(&path).is_ok() {
            return Err(());
        }

        let creation_time: u64 = SystemTime::now().duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs()).unwrap_or(0);
        let data = WorldData::new(name.to_string(), creation_time, seed);
        
        fs::create_dir(&path).unwrap();
        fs::write(&path.join("./world.json"), serde_json::to_vec_pretty(&data).unwrap()).unwrap();
        
        self.worlds.push(data);
        Ok(())
    }
}
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool};
use std::sync::{Mutex};

use serde::{Deserialize, Serialize};

use crate::bytes::{BytesCoder, cast_vec_from_bytes};
use crate::player::player::Player;
use crate::setting::Setting;
use crate::bytes::AsFromBytes;
use crate::voxels::new_chunk::Chunk;
use crate::voxels::new_chunks::{ChunkCoord, WORLD_HEIGHT};
use crate::UnsafeMutex;

// Must be a power of two
const REGION_SIZE: usize = 32;
const REGION_SIZE_BITS: usize = REGION_SIZE - 1;
const REGION_SQUARE: usize = REGION_SIZE*REGION_SIZE;
const REGION_BIT_SHIFT: usize = REGION_SIZE.ilog2() as usize;
const REGION_VOLUME: usize = REGION_SQUARE*WORLD_HEIGHT;

const REGION_MAGIC_NUMBER: u64 = 0x4474_304E_7AD7_835A;
const REGION_FORMAT_VERSION: u32 = 2;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegionFormatType {
    Region = 0,
    Blueprint = 1,
}

impl From<u8> for RegionFormatType {
    fn from(value: u8) -> Self {
        match value {
            1 => RegionFormatType::Blueprint,
            _ => RegionFormatType::Region
        }
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegionCoords(i32, i32);

impl RegionCoords {
    #[inline]
    pub fn filename(&self) -> String {
        self.0.to_string() + "_" + &self.1.to_string() + ".bin"
    }
}

impl From<ChunkCoord> for RegionCoords {
    fn from(c: ChunkCoord) -> Self {
        Self(c.x >> REGION_BIT_SHIFT, c.z >> REGION_BIT_SHIFT)
    }
}

pub trait RegionChunkIndex {
    fn region_chunk_index(&self) -> usize;
}

impl RegionChunkIndex for ChunkCoord {
    fn region_chunk_index(&self) -> usize {
        let x = self.x as usize & REGION_SIZE_BITS;
        let z = self.z as usize & REGION_SIZE_BITS;

        x * REGION_SIZE + z
    }
}

impl RegionChunkIndex for usize {
    fn region_chunk_index(&self) -> usize {
        *self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EncodedChunk{
    // Fields for special chunks can be added
    Some(Arc<[u8]>),
}

impl Clone for EncodedChunk {
    fn clone(&self) -> Self {
        match self {
            EncodedChunk::Some(d) => EncodedChunk::Some(Arc::clone(d)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Region {
    unsaved: AtomicBool,
    chunks: Mutex<HashMap<usize, EncodedChunk>>,
}

impl Region {
    pub fn new_empty() -> Self {
        Self { chunks: Mutex::new(HashMap::new()), unsaved: AtomicBool::new(false) }
    }

    pub fn chunk(&self, coords: impl RegionChunkIndex) -> Option<EncodedChunk> {
        self.chunks.lock().unwrap().get(&coords.region_chunk_index()).cloned()
    }

    pub fn save_chunk(&self, coords: impl RegionChunkIndex, encoded_chunk: EncodedChunk) {
        let index = coords.region_chunk_index();
        if index >= REGION_VOLUME {
            eprintln!("Incorrect coordinates!");
            return;
        };
        self.chunks.lock().unwrap().insert(index, encoded_chunk);
        self.set_unsaved(true);
    }

    pub fn unsaved(&self) -> bool {self.unsaved.load(Ordering::Acquire)}
    pub fn set_unsaved(&self, val: bool) {self.unsaved.store(val, Ordering::Release);}
}


#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldRegionsHeader {
    magic_number: u64,
    format_version: u32,
    format_type: RegionFormatType,
    width: u8,
    height: u8,
    depth: u8,
    region: Arc<Region>,
}

#[derive(Debug)]
pub struct WorldRegions {
    path: PathBuf,
    pub regions: Mutex<HashMap<RegionCoords, Arc<Region>>>
}

impl WorldRegions {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), regions: Mutex::new(HashMap::new()) }
    }

    pub fn chunk(&self, coords: ChunkCoord) -> Option<EncodedChunk> {
        self.get_or_create_region(coords.into())
            .chunk(coords)
    }


    pub fn save_chunk(&self, chunk: &Chunk) {
        self.get_or_create_region(chunk.coord.into())
            .save_chunk(chunk.coord, EncodedChunk::Some(chunk.encode_bytes().into()));
    }

    pub fn get_or_create_region(&self, coords: RegionCoords) -> Arc<Region> {
        let region = self.regions.lock().unwrap().get(&coords).cloned();
        if let Some(region) = region {return region};

        self.load_region(coords)
    }

    pub fn save_all_regions(&self) {
        let keys: Vec<RegionCoords> = self.regions.lock().unwrap().keys().cloned().collect();
        keys.iter().for_each(|key| self.save_region(*key));
    }

    pub fn save_region(&self, coords: RegionCoords) {
        let Some(region) = self.regions.lock().unwrap().get(&coords).cloned() else {return};
        if !region.unsaved() {return};
        let header = WorldRegionsHeader {
            magic_number: REGION_MAGIC_NUMBER,
            format_version: REGION_FORMAT_VERSION,
            format_type: RegionFormatType::Region,
            width: REGION_SIZE as u8,
            height: WORLD_HEIGHT as u8,
            depth: REGION_SIZE as u8,
            region: Arc::clone(&region)
        };

        let _ = fs::create_dir_all(self.path.join("regions/"));
        if let Err(err) = fs::write(self.path.join("regions/")
            .join(coords.filename()), bincode::serialize(&header).unwrap())
        {
            eprintln!("Region write error: {}", err);
        } else {
            region.set_unsaved(false);
        }
    }


    pub fn load_region(&self, coords: RegionCoords) -> Arc<Region> {
        let file = fs::read(self.path.join("regions/").join(coords.filename()));
        let region = if let Ok(bytes) = file {
            let header = bincode::deserialize::<WorldRegionsHeader>(&bytes).unwrap();
            header.region
        } else {
            Arc::new(Region::new_empty())
        };

        self.regions.lock().unwrap().insert(coords, Arc::clone(&region));
        region
    }
}


pub struct PlayerSave {
    path: PathBuf,
}

impl PlayerSave {
    pub fn new(path: PathBuf) -> Self {
        Self { path: path.join("player.bin") }
    }

    pub fn load_player(&self) -> Option<Player> {
        match fs::read(self.path.as_path()) {
            Ok(bytes) => Some(Player::decode_bytes(&bytes)),
            Err(_) => {None},
        }
    }

    pub fn save_player(&self, player: &Player) {
        if let Err(err) = fs::write(self.path.as_path(), player.encode_bytes()) {
            eprintln!("Player write error: {}", err);
        }
    }
}

pub struct WorldSaver {
    pub regions: Arc<WorldRegions>,
    pub player: Arc<UnsafeMutex<PlayerSave>>
}

impl WorldSaver {
    pub fn new(path: PathBuf) -> Self {
        Self {
            regions: Arc::new(WorldRegions::new(path.clone())),
            player: Arc::new(UnsafeMutex::new(PlayerSave::new(path))),
        }
    }
}

pub struct SettingSave {
    path: PathBuf,
}

impl SettingSave {
    pub fn new(path: PathBuf) -> Self {
        Self { path: path.join("setting.json") }
    }

    pub fn load(&self) -> Option<Setting> {
        match fs::read(self.path.as_path()) {
            Ok(bytes) => serde_json::from_slice(&bytes).ok(),
            Err(_) => {None},
        }
    }

    pub fn save(&self, setting: &Setting) {
        if let Err(err) = fs::write(self.path.as_path(), serde_json::to_vec_pretty(setting).unwrap()) {
            eprintln!("Player write error: {}", err);
        }
    }
}

pub struct Save {
    pub world: WorldSaver,
    pub setting: SettingSave,
}

impl Save {
    pub fn new(world_path: impl Into<PathBuf>, setting_path: impl Into<PathBuf>) -> Self {
        let path: PathBuf = world_path.into();
        std::fs::create_dir_all(path.join("regions/"))
            .expect("Error creating directory");
        Self { world: WorldSaver::new(path), setting: SettingSave::new(setting_path.into()) }
    }
}
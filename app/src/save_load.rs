use std::collections::HashMap;
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;

use crate::bytes::{BytesCoder, cast_vec_from_bytes};
use crate::player::player::Player;
use crate::setting::Setting;
use crate::voxels::chunk::Chunk;
use crate::voxels::chunks::WORLD_HEIGHT;
use crate::coords::chunk_coord::ChunkCoord;
use crate::bytes::AsFromBytes;
use crate::UnsafeMutex;

// Must be a power of two
const REGION_SIZE: usize = 32;
const REGION_SIZE_BITS: usize = REGION_SIZE - 1;
const REGION_SQUARE: usize = REGION_SIZE*REGION_SIZE;
const REGION_BIT_SHIFT: usize = REGION_SIZE.ilog2() as usize;
const REGION_VOLUME: usize = REGION_SQUARE*WORLD_HEIGHT;

const REGION_MAGIC_NUMBER: u64 = 0x4474_304E_7AD7_835A;
const REGION_FORMAT_VERSION: u32 = 2;

const NONE_ENCODED_CHUNK: EncodedChunk = EncodedChunk::None;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
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
        let y = self.y as usize & (WORLD_HEIGHT - 1);
        let z = self.z as usize & REGION_SIZE_BITS;

        (x * WORLD_HEIGHT + y) * REGION_SIZE + z
    }
}

#[derive(Debug, Default)]
pub enum EncodedChunk{
    // Fields for special chunks can be added
    #[default]
    None,
    Some(Box<[u8]>),
}

#[derive(Debug)]
pub struct Region {
    unsaved: bool,
    chunks: Box<[EncodedChunk; REGION_VOLUME]>
}

impl Region {
    pub fn new_empty() -> Self {
        Self { chunks: Box::new([NONE_ENCODED_CHUNK; REGION_VOLUME]), unsaved: false }
    }

    pub fn chunk(&self, coords: impl RegionChunkIndex) -> &EncodedChunk {
        &self.chunks[coords.region_chunk_index()]
    }

    pub fn save_chunk(&mut self, coords: impl RegionChunkIndex, data: Box<[u8]>) {
        if let Some(chunk) = self.chunks.get_mut(coords.region_chunk_index()) {
            *chunk = EncodedChunk::Some(data);
            self.unsaved = true;
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WorldRegionsHeader {
    magic_number: u64,
    format_version: u32,
    format_type: RegionFormatType,
    width: u8,
    height: u8,
    depth: u8,
}
impl AsFromBytes for WorldRegionsHeader {}

#[derive(Debug)]
pub struct WorldRegions {
    path: PathBuf,
    pub regions: HashMap<RegionCoords, Region>
}

impl WorldRegions {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), regions: HashMap::new() }
    }

    pub fn chunk(&mut self, coords: ChunkCoord) -> &EncodedChunk {
        self.get_or_create_region(coords.into())
            .chunk(coords)
    }


    pub fn save_chunk(&mut self, chunk: &Chunk) {
        self.get_or_create_region(chunk.xyz.into())
            .save_chunk(chunk.xyz, chunk.encode_bytes());
    }

    pub fn get_or_create_region(&mut self, coords: RegionCoords) -> &mut Region {
        let self_ptr = self as *mut Self;
        if let Some(region) = self.regions.get_mut(&coords) {
            return region;
        }
        unsafe { self_ptr.as_mut().unwrap() }.load_region(coords);
        unsafe { self_ptr.as_mut().unwrap() }.regions.get_mut(&coords).unwrap()
    }

    pub fn save_all_regions(&mut self) {
        let self_ptr = self as *mut Self;
        self.regions.keys().for_each(|key| unsafe { self_ptr.as_mut().unwrap() }.save_region(*key));
    }

    pub fn save_region(&mut self, coords: RegionCoords) {
        let Some(region) = self.regions.get_mut(&coords) else {return};
        if !region.unsaved {return};
        let header = WorldRegionsHeader {
            magic_number: REGION_MAGIC_NUMBER,
            format_version: REGION_FORMAT_VERSION,
            format_type: RegionFormatType::Region,
            width: REGION_SIZE as u8,
            height: WORLD_HEIGHT as u8,
            depth: REGION_SIZE as u8,
        };


        let mut bytes = Vec::<u8>::new();
        bytes.extend(header.as_bytes());
        region.chunks.iter()
            .map(|chunk| if let EncodedChunk::Some(b) = chunk {b.len() as u32} else {0})
            .for_each(|b| bytes.extend(b.as_bytes()));
        region.chunks.iter()
            .filter_map(|chunk| if let EncodedChunk::Some(b) = chunk {Some(b)} else {None})
            .for_each(|b| bytes.extend(b.as_ref()));

        let _ = fs::create_dir_all(self.path.join("regions/"));
        if let Err(err) = fs::write(self.path.join("regions/").join(coords.filename()), bytes) {
            eprintln!("Region write error: {}", err);
        } else {
            region.unsaved = false;
        }
    }


    pub fn load_region(&mut self, coords: RegionCoords) {
        let mut region = Region::new_empty();
        let file = fs::read(self.path.join("regions/").join(coords.filename()));
        if let Ok(bytes) = file {
            let header = WorldRegionsHeader::from_bytes(&bytes[0..WorldRegionsHeader::size()]);
            let volume = header.width as usize * header.height as usize * header.width as usize;
            let offsets_end = WorldRegionsHeader::size()+std::mem::size_of::<u32>()*volume;
            let offsets = cast_vec_from_bytes::<u32>(&bytes[WorldRegionsHeader::size()..offsets_end]);
            let mut chunk_offset = offsets_end;
            for (i, offset) in offsets.into_iter().enumerate().filter(|(_, offset)| *offset != 0) {
                let Some(chunk) = region.chunks.get_mut(i) else {continue};
                *chunk = EncodedChunk::Some(bytes[chunk_offset..offset as usize+chunk_offset].into());
                chunk_offset += offset as usize;
            }
        }

        self.regions.insert(coords, region);
    }

    /// Only safe access in an UnsafeMutex
    pub fn change_path(&mut self, path: PathBuf) {
        self.path = path;
        self.regions = HashMap::new();
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
    pub regions: Arc<UnsafeMutex<WorldRegions>>,
    pub player: Arc<UnsafeMutex<PlayerSave>>
}

impl WorldSaver {
    pub fn new(path: PathBuf) -> Self {
        Self {
            regions: Arc::new(UnsafeMutex::new(WorldRegions::new(path.clone()))),
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
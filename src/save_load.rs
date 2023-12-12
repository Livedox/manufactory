use std::collections::HashMap;
use std::{default, fs};
use std::path::PathBuf;
use std::{path::Path, fs::File};
use std::io::prelude::*;

use zerocopy::FromBytes;
use zerocopy_derive::{FromBytes, FromZeroes, AsBytes};

use crate::bytes::DynByteInterpretation;
use crate::voxels::chunk::Chunk;
use crate::voxels::chunks::WORLD_HEIGHT;
use crate::world::chunk_coords::ChunkCoords;
use crate::bytes::NumFromBytes;

// Must be a power of two
const REGION_SIZE: usize = 32;
const REGION_SIZE_BITS: usize = REGION_SIZE - 1;
const REGION_SQUARE: usize = REGION_SIZE*REGION_SIZE;
const REGION_BIT_SHIFT: usize = REGION_SIZE.ilog2() as usize;
const REGION_VOLUME: usize = REGION_SQUARE*WORLD_HEIGHT;

const REGION_MAGIC_NUMBER: u64 = 0x4474_304E_7AD7_835A;
const REGION_FORMAT_VERSION: u32 = 1;

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

impl From<ChunkCoords> for RegionCoords {
    fn from(c: ChunkCoords) -> Self {
        Self(c.0 >> REGION_BIT_SHIFT, c.2 >> REGION_BIT_SHIFT)
    }
}

pub trait RegionChunkIndex {
    fn region_chunk_index(&self) -> usize;
}

impl RegionChunkIndex for ChunkCoords {
    fn region_chunk_index(&self) -> usize {
        let x = self.0 as usize & REGION_SIZE_BITS;
        let y = self.1 as usize & (WORLD_HEIGHT - 1);
        let z = self.2 as usize & REGION_SIZE_BITS;

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

#[derive(Debug)]
pub struct WorldRegions {
    path: PathBuf,
    pub regions: HashMap<RegionCoords, Region>
}

impl WorldRegions {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), regions: HashMap::new() }
    }

    pub fn chunk(&mut self, coords: ChunkCoords) -> &EncodedChunk {
        self.get_or_create_region(coords.into())
            .chunk(coords)
    }


    pub fn save_chunk(&mut self, chunk: &Chunk) {
        self.get_or_create_region(chunk.xyz.into())
            .save_chunk(chunk.xyz, chunk.to_bytes());
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
        let mut bytes = Vec::<u8>::new();
        bytes.extend(REGION_MAGIC_NUMBER.to_le_bytes());
        bytes.extend(REGION_FORMAT_VERSION.to_le_bytes());
        bytes.extend((RegionFormatType::Region as u8).to_le_bytes());
        bytes.extend((REGION_SIZE as u8).to_le_bytes());
        bytes.extend((WORLD_HEIGHT as u8).to_le_bytes());
        bytes.extend((REGION_SIZE as u8).to_le_bytes());

        for chunk in region.chunks.iter() {
            if let EncodedChunk::Some(b) = chunk {
                bytes.extend((b.len() as u32).to_le_bytes());
            } else {
                bytes.extend(0u32.to_le_bytes());
            }
        }
        for chunk in region.chunks.iter() {
            if let EncodedChunk::Some(b) = chunk {
                bytes.extend(b.as_ref());
            }
        }

        if let Err(err) = fs::write(self.path.join(&coords.filename()), bytes) {
            eprintln!("Region write error: {}", err);
        } else {
            region.unsaved = false;
        }
    }


    pub fn load_region(&mut self, coords: RegionCoords) {
        let mut region = Region::new_empty();
        if let Ok(bytes) = fs::read(self.path.join(&coords.filename())) {
            let mut offsets = Vec::<usize>::new();
            for num in bytes[16..16+REGION_VOLUME*4].chunks(4).into_iter() {
                offsets.push(u32::from_bytes(num) as usize);
            }
            let mut chunk_offset = 16+REGION_VOLUME*4;
            for (i, offset) in offsets.into_iter().enumerate() {
                if offset == 0 {continue};
                let Some(chunk) = region.chunks.get_mut(i) else {continue};
                *chunk = EncodedChunk::Some(bytes[chunk_offset..offset+chunk_offset].into());
                chunk_offset += offset;
            }
        }

        self.regions.insert(coords, region);
        println!("{:?} {:?} {:?}", self.path, self.path.join(&coords.filename()), Path::new(&coords.filename()));
    }
}

pub trait Compress: Sized + Clone {
    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        let len = std::mem::size_of_val(self);
        let slf: *const Self = self;
        unsafe { std::slice::from_raw_parts(slf.cast::<u8>(), len) }
    }

    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Self {
        let ptr = bytes.as_ptr() as *const Self;
        unsafe {ptr.as_ref()}.unwrap().clone()
    }
}
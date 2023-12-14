use std::{collections::HashMap, rc::Rc, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, cell::UnsafeCell, ops::{Deref, DerefMut}, time::{SystemTime, UNIX_EPOCH}, io::Cursor, mem::MaybeUninit};

use itertools::iproduct;
use bitflags::bitflags;
use crate::{light::light_map::LightMap, direction::Direction, world::{local_coords::LocalCoords, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, GAME_VERSION, engine::pipeline::new, bytes::{AsFromBytes, BytesCoder}};

use super::{voxel::{self, Voxel}, voxel_data::{VoxelData, VoxelAdditionalData, self}, chunks::Chunks, block::blocks::BLOCKS};
use std::io::prelude::*;
use flate2::{Compression, read::ZlibDecoder};
use flate2::write::ZlibEncoder;

pub const CHUNK_SIZE: usize = 32;
pub const HALF_CHUNK_SIZE: usize = CHUNK_SIZE/2;
pub const CHUNK_SQUARE: usize = CHUNK_SIZE.pow(2);
pub const CHUNK_VOLUME: usize = CHUNK_SIZE.pow(3);
pub const CHUNK_BIT_SHIFT: usize = CHUNK_SIZE.ilog2() as usize;
pub const CHUNK_BITS: usize = CHUNK_SIZE - 1_usize;
pub const COMPRESSION_TYPE: CompressionType = CompressionType::ZLIB;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CompressionType {
    NONE = 0b000000,
    ZLIB = 0b000001,
}

impl From<u8> for CompressionType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NONE,
            1 => Self::ZLIB,
            _ => Self::NONE
        }
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub voxels: [voxel::Voxel; CHUNK_VOLUME],
    pub voxels_data: HashMap<usize, VoxelData>,
    modified: AtomicBool,
    pub unsaved: bool,
    pub lightmap: LightMap,
    pub xyz: ChunkCoords,
}


impl Chunk {
    pub fn new(pos_x: i32, pos_y: i32, pos_z: i32) -> Chunk {
        let mut voxels = [Voxel::new(0); CHUNK_VOLUME];
        let voxels_data = HashMap::new();

        for (y, z, x) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
            let real_x = x as i32 + pos_x*CHUNK_SIZE as i32;
            let real_y = y as i32 + pos_y*CHUNK_SIZE as i32;
            let real_z = z as i32 + pos_z*CHUNK_SIZE as i32;
            if real_y as f64 <= ((real_x as f64 *0.3).sin() * 0.5 + 0.5) * 10. {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 7;
            }
            if real_y <= 2 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 5;
            }
            if z == 0 && y == 16 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 7;
            }
            if x == 0 && y == 0 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 7;
            }

            if real_z == 200 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 7;
            }
        }

        Chunk {
            voxels,
            xyz: ChunkCoords(pos_x, pos_y, pos_z),
            voxels_data,
            unsaved: true,
            modified: AtomicBool::new(true),
            lightmap: LightMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|voxel| { voxel.id == 0 })
    }


    pub fn is_air(&self, coords: LocalCoords) -> bool {
        self.voxel(coords).id == 0
    }

    pub fn voxel(&self, local_coords: LocalCoords) -> &Voxel {
        &self.voxels[local_coords.index()]
    }

    fn mut_voxel(&mut self, local_coords: LocalCoords) -> &mut Voxel {
        &mut self.voxels[local_coords.index()]
    }

    pub fn modified(&self) -> bool {
        self.modified.load(Ordering::Acquire)
    }

    pub fn modify(&mut self, value: bool) {
        self.modified.store(value, Ordering::Release);
    }

    pub fn set_voxel_id(&mut self, local_coords: LocalCoords, id: u32, direction: Option<&Direction>) {
        self.voxels_data.remove(&local_coords.index());
        self.mut_voxel(local_coords).id = id;
        if BLOCKS()[id as usize].is_additional_data() {
            self.voxels_data.insert(local_coords.index(), VoxelData {
                id,
                global_coords: self.xyz.to_global(local_coords),
                additionally: Arc::new(VoxelAdditionalData::new(id, direction.unwrap_or(&Direction::new_x()))),
            });
        }
    }


    pub fn mut_voxel_data(&mut self, local_coords: LocalCoords) -> Option<&mut VoxelData> {
        self.voxels_data.get_mut(&local_coords.index())
    }


    pub fn voxel_data(&self, local_coords: LocalCoords) -> Option<&VoxelData> {
        self.voxels_data.get(&local_coords.index())
    }


    pub fn voxels_data(&self) -> &HashMap<usize, VoxelData> {
        &self.voxels_data
    }


    pub fn mut_voxels_data(&mut self) -> &mut HashMap<usize, VoxelData> {
        &mut self.voxels_data
    }


    pub fn add_voxel_data(&mut self, local_coords: LocalCoords, voxel_data: VoxelData) -> Option<VoxelData> {
        self.voxels_data.insert(local_coords.index(), voxel_data)
    }
}

impl AsFromBytes for [Voxel; CHUNK_VOLUME] {}
impl BytesCoder for [Voxel; CHUNK_VOLUME] {
    fn decode_bytes(bytes: &[u8]) -> Self {
        let mut decoder = ZlibDecoder::new(bytes);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        <[Voxel; CHUNK_VOLUME]>::from_bytes(&buf)
    }

    fn encode_bytes(&self) -> Box<[u8]> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(self.as_bytes()).unwrap();
        encoder.finish().unwrap().into()
    }
}


impl BytesCoder for HashMap<usize, VoxelData> {
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        if !self.is_empty() {
            for (key, val) in self.iter() {
                bytes.extend((*key as u32).as_bytes());
                let encode_data = val.encode_bytes();
                bytes.extend((encode_data.len() as u32).as_bytes());
                bytes.extend(encode_data.as_ref());
            }
        }

        bytes.into()
    }

    fn decode_bytes(bytes: &[u8]) -> Self {
        let mut h = HashMap::<usize, VoxelData>::new();
        let mut offset: usize = 0;
        while offset < bytes.len() {
            let key_end = offset as usize + u32::size();
            let key = u32::from_bytes(&bytes[offset..key_end]) as usize;
            let len_end = key_end+u32::size();
            let len = u32::from_bytes(&bytes[key_end..len_end]) as usize;
            let vd = VoxelData::decode_bytes(&bytes[len_end..len_end+len]);
            h.insert(key, vd);
            offset = len_end+len;
        }
        h
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CompressChunk {
    pub time: u64,
    pub xyz: ChunkCoords,
    pub voxel_len: u32,
    pub voxel_data_len: u32,
    pub compression_type: CompressionType,
}
impl AsFromBytes for CompressChunk {}

impl BytesCoder for Chunk {
    fn encode_bytes(&self) -> Box<[u8]> {
        let voxels = self.voxels.encode_bytes();
        let voxel_data = self.voxels_data.encode_bytes();
        let compress = CompressChunk {
            time: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
            xyz: self.xyz,
            voxel_len: voxels.len() as u32,
            voxel_data_len: voxel_data.len() as u32,
            compression_type: COMPRESSION_TYPE,
        };
        
        let mut bytes = Vec::new();
        bytes.extend(compress.as_bytes());
        bytes.extend(voxels.as_ref());
        bytes.extend(voxel_data.as_ref());
        bytes.into()
    }
    fn decode_bytes(data: &[u8]) -> Self {
        let compress = CompressChunk::from_bytes(&data[0..CompressChunk::size()]);
        let voxel_end = CompressChunk::size() + compress.voxel_len as usize;
        let voxel_data_end = voxel_end + compress.voxel_data_len as usize;
        let voxels = <[Voxel; CHUNK_VOLUME]>::decode_bytes(&data[CompressChunk::size()..voxel_end]);
        let voxels_data = <HashMap::<usize, VoxelData>>::decode_bytes(&data[voxel_end..voxel_data_end]);

        Self {
            voxels,
            voxels_data,
            modified: AtomicBool::new(true),
            unsaved: false,
            lightmap: LightMap::new(),
            xyz: compress.xyz,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::voxels::chunk::CHUNK_SIZE;

    #[test]
    fn correct_chunk_size() {
        assert!(CHUNK_SIZE > 1 && (CHUNK_SIZE & CHUNK_SIZE-1) == 0 && CHUNK_SIZE <= 32);
    }
}
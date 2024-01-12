use std::{collections::HashMap, sync::{Arc, atomic::{AtomicBool, Ordering}, RwLock}, time::{SystemTime, UNIX_EPOCH}};

use itertools::iproduct;
use crate::{light::light_map::{LightMap, Light}, direction::Direction, world::{local_coords::LocalCoords, chunk_coords::ChunkCoords}, bytes::{AsFromBytes, BytesCoder}};

use super::{voxel::{self, Voxel, VoxelAtomic}, voxel_data::{VoxelData, VoxelAdditionalData}, block::blocks::BLOCKS};
use std::io::prelude::*;
use flate2::{Compression, read::ZlibDecoder};
use flate2::write::ZlibEncoder;

pub const CHUNK_SIZE: usize = 32;
pub const HALF_CHUNK_SIZE: usize = CHUNK_SIZE/2;
pub const _CHUNK_SQUARE: usize = CHUNK_SIZE.pow(2);
pub const CHUNK_VOLUME: usize = CHUNK_SIZE.pow(3);
pub const CHUNK_BIT_SHIFT: usize = CHUNK_SIZE.ilog2() as usize;
pub const CHUNK_BITS: usize = CHUNK_SIZE - 1_usize;
pub const COMPRESSION_TYPE: CompressionType = CompressionType::Zlib;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CompressionType {
    None = 0b000000,
    Zlib = 0b000001,
}

impl From<u8> for CompressionType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Zlib,
            _ => Self::None
        }
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub voxels: [VoxelAtomic; CHUNK_VOLUME],
    pub lightmap: LightMap,
    
    pub voxels_data: Arc<RwLock<HashMap<usize, Arc<VoxelData>>>>,
    modified: AtomicBool,
    unsaved: AtomicBool,
    pub xyz: ChunkCoords,
}


impl Chunk {
    pub fn new(pos_x: i32, pos_y: i32, pos_z: i32) -> Chunk {
        let voxels: [VoxelAtomic; CHUNK_VOLUME] = unsafe {std::mem::zeroed()};

        for (y, z, x) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
            let real_x = x as i32 + pos_x*CHUNK_SIZE as i32;
            let real_y = y as i32 + pos_y*CHUNK_SIZE as i32;
            let real_z = z as i32 + pos_z*CHUNK_SIZE as i32;

            if real_y as f64 <= ((real_x as f64 *0.3).sin() * 0.5 + 0.5) * 10. {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].update(7);
            }
            if real_y <= 2 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].update(5);
            }
            if z == 0 && y == 16 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].update(7);
            }
            if x == 0 && y == 0 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].update(5);
            }

            // if real_z == 200 {
            //     voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 7;
            // }
        }

        Chunk {
            voxels,
            xyz: ChunkCoords(pos_x, pos_y, pos_z),
            voxels_data: Arc::new(RwLock::new(HashMap::new())),
            unsaved: AtomicBool::new(true),
            modified: AtomicBool::new(true),
            lightmap: LightMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|voxel| { voxel.id() == 0 })
    }


    pub fn is_air(&self, coords: LocalCoords) -> bool {
        self.voxel(coords).id == 0
    }

    pub unsafe fn get_unchecked_voxel(&self, local_coords: LocalCoords) -> Voxel {
        self.voxels.get_unchecked(local_coords.index()).to_voxel()
    }

    #[inline]
    pub fn voxel_id(&self, local_coords: LocalCoords) -> u32 {
        self.voxels[local_coords.index()].id()
    }

    pub fn voxel(&self, local_coords: LocalCoords) -> Voxel {
        self.voxels[local_coords.index()].to_voxel()
    }

    pub fn set_voxel(&self, local_coords: LocalCoords, id: u32) {
        self.voxels[local_coords.index()].update(id);
    }

    #[inline]
    pub fn modified(&self) -> bool {
        self.modified.load(Ordering::Acquire)
    }

    #[inline]
    pub fn modify(&self, value: bool) {
        self.modified.store(value, Ordering::Release);
    }

    #[inline]
    pub fn unsaved(&self) -> bool {
        self.unsaved.load(Ordering::Acquire)
    }

    #[inline]
    pub fn save(&self, value: bool) {
        self.unsaved.store(value, Ordering::Release);
    }

    pub fn set_voxel_id(&self, local_coords: LocalCoords, id: u32, direction: Option<&Direction>) {
        self.voxels_data.write().unwrap().remove(&local_coords.index());
        self.set_voxel(local_coords, id);
        if BLOCKS()[id as usize].is_additional_data() {
            self.voxels_data.write().unwrap().insert(local_coords.index(), Arc::new(VoxelData {
                id,
                global_coords: self.xyz.to_global(local_coords),
                additionally: Arc::new(VoxelAdditionalData::new(id, direction.unwrap_or(&Direction::new_x()))),
            }));
        }
    }

    pub fn voxel_data(&self, local_coords: LocalCoords) -> Option<Arc<VoxelData>> {
        self.voxels_data.read().unwrap().get(&local_coords.index()).map(|d| d.clone())
    }


    pub fn voxels_data(&self) -> Arc<RwLock<HashMap<usize, Arc<VoxelData>>>> {
        self.voxels_data.clone()
    }


    pub fn add_voxel_data(&self, local_coords: LocalCoords, voxel_data: VoxelData) {
        self.voxels_data.write().unwrap().insert(local_coords.index(), Arc::new(voxel_data));
    }

    #[inline]
    pub fn get_light(&self, local_coords: LocalCoords) -> Light {
        self.lightmap.get_light(local_coords.into())
    }

    #[inline]
    pub fn get_light_channel(&self, local_coords: LocalCoords, channel: usize) -> u8 {
        self.lightmap.get(local_coords.into()).get_channel(channel)
    }

    #[inline]
    pub unsafe fn get_unchecked_light_channel(&self, local_coords: LocalCoords, channel: usize) -> u8 {
        unsafe {self.lightmap.get_unchecked_channel(local_coords.into(), channel)}
    }
}

pub trait Shit: Sized {
    const SIZE: usize = std::mem::size_of::<Self>();
    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        let slf: *const Self = self;
        unsafe { std::slice::from_raw_parts(slf.cast::<u8>(), Self::SIZE) }
    }

    #[inline]
    fn from_bytes(bytes: Box<[u8]>) -> Self {
        assert_eq!(bytes.len(), Self::SIZE);
        let ptr = Box::into_raw(bytes) as *mut Self;
        *unsafe {Box::from_raw(ptr)}
    }

    #[inline(always)]
    fn size() -> usize {Self::SIZE}
}

impl Shit for [VoxelAtomic; CHUNK_VOLUME] {}

impl BytesCoder for [VoxelAtomic; CHUNK_VOLUME] {
    fn decode_bytes(bytes: &[u8]) -> Self {
        let mut decoder = ZlibDecoder::new(bytes);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        <[VoxelAtomic; CHUNK_VOLUME]>::from_bytes(buf.into())
    }

    fn encode_bytes(&self) -> Box<[u8]> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(self.as_bytes()).unwrap();
        encoder.finish().unwrap().into()
    }
}


impl BytesCoder for Arc<RwLock<HashMap<usize, Arc<VoxelData>>>> {
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        if !self.read().unwrap().is_empty() {
            for (key, val) in self.read().unwrap().iter() {
                bytes.extend((*key as u32).as_bytes());
                let encode_data = val.encode_bytes();
                bytes.extend((encode_data.len() as u32).as_bytes());
                bytes.extend(encode_data.as_ref());
            }
        }

        bytes.into()
    }

    fn decode_bytes(bytes: &[u8]) -> Self {
        let mut h = HashMap::<usize, Arc<VoxelData>>::new();
        let mut offset: usize = 0;
        while offset < bytes.len() {
            let key_end = offset + u32::size();
            let key = u32::from_bytes(&bytes[offset..key_end]) as usize;
            let len_end = key_end+u32::size();
            let len = u32::from_bytes(&bytes[key_end..len_end]) as usize;
            let vd = VoxelData::decode_bytes(&bytes[len_end..len_end+len]);
            h.insert(key, Arc::new(vd));
            offset = len_end+len;
        }
        Arc::new(RwLock::new(h))
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
        let voxels = <[VoxelAtomic; CHUNK_VOLUME]>::decode_bytes(&data[CompressChunk::size()..voxel_end]);
        let voxels_data = <Arc<RwLock<HashMap<usize, Arc<VoxelData>>>>>::decode_bytes(&data[voxel_end..voxel_data_end]);

        Self {
            voxels,
            voxels_data,
            modified: AtomicBool::new(true),
            unsaved: AtomicBool::new(false),
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
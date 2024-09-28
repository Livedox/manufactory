use std::{collections::HashMap, ops::Index, slice::SliceIndex, sync::{atomic::{AtomicBool, Ordering}, Arc, RwLock}, time::{SystemTime, UNIX_EPOCH}};

use itertools::iproduct;
use serde::{de::Visitor, ser::{SerializeStruct, SerializeTuple, SerializeTupleStruct}, Deserialize, Serialize};
use crate::{bytes::{AsFromBytes, BytesCoder}, content::{Content}, light::new_light_map::{LightMap}, light::new_light::{Light}};

use super::{generator::Generator, live_voxels::LiveVoxelContainer, new_chunks::{ChunkCoord, WORLD_BLOCK_HEIGHT}, voxel::{Voxel, VoxelAtomic}};
use std::io::prelude::*;
use flate2::{Compression, read::ZlibDecoder};
use flate2::write::ZlibEncoder;

pub const CHUNK_SIZE: usize = 32;
pub const HALF_CHUNK_SIZE: usize = CHUNK_SIZE/2;
pub const CHUNK_SQUARE: usize = CHUNK_SIZE.pow(2);
pub const CHUNK_VOLUME: usize = CHUNK_SIZE.pow(3) * 8;
pub const CHUNK_BIT_SHIFT: usize = CHUNK_SIZE.ilog2() as usize;
pub const CHUNK_BITS: usize = CHUNK_SIZE - 1_usize;
pub const COMPRESSION_TYPE: CompressionType = CompressionType::Zlib;
pub const IS_LITTLE_ENDIAN: bool = cfg!(target_endian = "little");

#[derive(Debug, Clone, Copy)]
pub struct LocalCoord {pub x: usize, pub y: usize, pub z: usize}
impl LocalCoord {
    #[inline]
    pub fn new(x: usize, y: usize, z: usize) -> Self {Self { x, y, z }}

    #[inline]
    pub fn index(&self) -> usize {
        (self.y * CHUNK_SIZE + self.z)*CHUNK_SIZE + self.x
    }
}

impl From<(usize, usize, usize)> for LocalCoord {
    fn from(value: (usize, usize, usize)) -> Self {
        Self { x: value.0, y: value.1, z: value.2 }
    }
}

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

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct LiveVoxels(pub Arc<RwLock<HashMap<usize, Arc<LiveVoxelContainer>>>>);

impl LiveVoxels {
    pub fn get(&self, k: &usize) -> Option<Arc<LiveVoxelContainer>> {
        self.0.read().unwrap().get(k).cloned()
    }

    pub fn insert(&self, k: usize, v: Arc<LiveVoxelContainer>) -> Option<Arc<LiveVoxelContainer>> {
        self.0.write().unwrap().insert(k, v)
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().unwrap().is_empty()
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Voxels(pub [VoxelAtomic; CHUNK_VOLUME]);

impl Default for Voxels {
    fn default() -> Self {
        unsafe {std::mem::zeroed()}
    }
}

impl Index<LocalCoord> for Voxels {
    type Output = VoxelAtomic;
    fn index(&self, index: LocalCoord) -> &Self::Output {
        &self.0[index.index()]
    }
}

impl Voxels {
    #[inline]
    pub fn get(&self, lc: LocalCoord) -> Option<&VoxelAtomic> {
        self.0.get(lc.index())
    }
    #[inline]
    pub unsafe fn get_unchecked(&self, lc: LocalCoord) -> &VoxelAtomic {
        unsafe {self.0.get_unchecked(lc.index())}
    }
    #[inline]
    pub fn inner(&self) -> &[VoxelAtomic; CHUNK_VOLUME] {
        &self.0
    }
}


#[derive(Debug)]
pub struct Chunk {
    pub voxels: Voxels,
    pub lightmap: LightMap,
    
    pub live_voxels: LiveVoxels,
    modified: AtomicBool,
    unsaved: AtomicBool,
    pub coord: ChunkCoord,
}


impl Chunk {
    pub fn new(generator: &Generator, cx: i32, cz: i32) -> Chunk {
        let voxels = Voxels::default();

        for (y, z, x) in iproduct!(0..WORLD_BLOCK_HEIGHT, 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
            let real_x = x as i32 + cx*CHUNK_SIZE as i32;
            let real_y = y as i32;
            let real_z = z as i32 + cz*CHUNK_SIZE as i32;

            let id = generator.generate(real_x, real_y, real_z);
            voxels.0[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].update(id);
        }

        Chunk {
            voxels,
            coord: ChunkCoord::new(cx, cz),
            live_voxels: LiveVoxels(Arc::new(RwLock::new(HashMap::new()))),
            unsaved: AtomicBool::new(true),
            modified: AtomicBool::new(true),
            lightmap: LightMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.0.iter().all(|voxel| { voxel.id() == 0 })
    }


    pub fn is_air(&self, lc: LocalCoord) -> bool {
        self.voxels[lc].id() == 0
    }

    pub fn voxels(&self) -> &Voxels {&self.voxels}

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

    pub fn set_voxel_id(&self, lc: LocalCoord, id: u32) {
        self.live_voxels.0.write().unwrap().remove(&lc.index());
        self.voxels[lc].update(id);
    }

    pub fn live_voxel(&self, local_coords: LocalCoord) -> Option<Arc<LiveVoxelContainer>> {
        self.live_voxels.get(&local_coords.index())
    }

    pub fn live_voxels(&self) -> LiveVoxels {
        self.live_voxels.clone()
    }

    pub fn light_map(&self) -> &LightMap {
        &self.lightmap
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

impl Serialize for Voxels {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        let mut voxels = serializer.serialize_tuple(2)?;
        voxels.serialize_element(&IS_LITTLE_ENDIAN)?;
        let size = std::mem::size_of::<Self>();
        let ptr: *const Self = self;
        let slice = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), size) };
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(slice).unwrap();
        let v = encoder.finish().unwrap();
        voxels.serialize_element(&v)?;
        voxels.end()
    }
}

impl<'de> Deserialize<'de> for Voxels {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>
    {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = (bool, Vec<u8>);
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
            {
                let is_le = seq.next_element::<bool>()?.unwrap();
                let bytes = seq.next_element::<Vec<u8>>()?.unwrap();

                Ok((is_le, bytes))
            }
            
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                todo!()
            }
        }

        let tuple = deserializer.deserialize_tuple(2, V)?;

        let mut decoder = ZlibDecoder::new(&tuple.1[..]);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        let voxels = <[VoxelAtomic; CHUNK_VOLUME]>::from_bytes(buf.into());

        Ok(Voxels(voxels))
    }
}

impl LiveVoxels {
    fn encode_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        if !self.0.read().unwrap().is_empty() {
            for (key, val) in self.0.read().unwrap().iter() {
                bytes.extend((*key as u32).as_bytes());
                let encode_data = val.to_bytes();
                bytes.extend((encode_data.len() as u32).as_bytes());
                bytes.extend(encode_data);
            }
        }

        bytes.into()
    }

    fn decode_bytes(content: &Content, bytes: &[u8]) -> Self {
        let mut h = HashMap::<usize, Arc<LiveVoxelContainer>>::new();
        let mut offset: usize = 0;
        while offset < bytes.len() {
            let key_end = offset + u32::size();
            let key = u32::from_bytes(&bytes[offset..key_end]) as usize;
            let len_end = key_end+u32::size();
            let len = u32::from_bytes(&bytes[key_end..len_end]) as usize;
            let vd = LiveVoxelContainer::from_bytes(content, &bytes[len_end..len_end+len]);
            h.insert(key, Arc::new(vd));
            offset = len_end+len;
        }
        Self(Arc::new(RwLock::new(h)))
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CompressChunk {
    pub time: u64,
    pub coord: ChunkCoord,
    pub voxel_len: u32,
    pub voxel_data_len: u32,
    pub compression_type: CompressionType,
}
impl AsFromBytes for CompressChunk {}

impl Chunk {
    pub fn encode_bytes(&self) -> Box<[u8]> {
        let voxels = bincode::serialize(&self.voxels).unwrap();
        let voxel_data = self.live_voxels.encode_bytes();
        let compress = CompressChunk {
            time: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
            coord: self.coord,
            voxel_len: voxels.len() as u32,
            voxel_data_len: voxel_data.len() as u32,
            compression_type: COMPRESSION_TYPE,
        };
        
        let mut bytes = Vec::new();
        bytes.extend(compress.as_bytes());
        bytes.extend(voxels);
        bytes.extend(voxel_data.as_ref());
        bytes.into()
    }

    
    pub fn decode_bytes(content: &Content, data: &[u8]) -> Self {
        let compress = CompressChunk::from_bytes(&data[0..CompressChunk::size()]);
        let voxel_end = CompressChunk::size() + compress.voxel_len as usize;
        let voxel_data_end = voxel_end + compress.voxel_data_len as usize;
        let voxels: Voxels = bincode::deserialize(&data[CompressChunk::size()..voxel_end]).unwrap();
        let live_voxels = LiveVoxels::decode_bytes(content, &data[voxel_end..voxel_data_end]);

        Self {
            voxels,
            live_voxels,
            modified: AtomicBool::new(true),
            unsaved: AtomicBool::new(false),
            lightmap: LightMap::new(),
            coord: compress.coord
        }
    }
}
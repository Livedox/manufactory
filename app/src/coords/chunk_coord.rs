use crate::voxels::chunk::CHUNK_SIZE;
use std::hash::{BuildHasher, Hash, Hasher};
use super::{global_coord::GlobalCoord, local_coord::LocalCoord};

#[repr(C)]
pub struct MyHasher {
    pub x: i32,
    pub z: i32
}

impl MyHasher {
    pub fn new() -> Self {Self {x: 0, z: 0}}
}

impl Hasher for MyHasher {
    #[inline]
    fn finish(&self) -> u64 {
        (self.x * 67_108_864 + self.z) as u64//1 875 000
    }

    fn write(&mut self, _bytes: &[u8]) {
        todo!();
    }

    #[inline]
    fn write_u64(&mut self, u: u64) {
        *self = unsafe {std::mem::transmute::<_, Self>(u)}
    }
}

impl BuildHasher for MyHasher {
    type Hasher = MyHasher;

    fn build_hasher(&self) -> Self::Hasher {
        MyHasher {x: 0, z: 0}
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32
}

impl ChunkCoord {
    pub const fn new(x: i32, z: i32) -> Self {Self { x, z }}

    #[inline]
    pub const fn to_global(self, local: LocalCoord) -> GlobalCoord {
        GlobalCoord::new(
            self.x * CHUNK_SIZE as i32 + local.x as i32, 
            local.y as i32, 
            self.z * CHUNK_SIZE as i32 + local.z as i32)
    }
}

impl From<(i32, i32)> for ChunkCoord {
    fn from(value: (i32, i32)) -> Self {
        Self { x: value.0, z: value.1 }
    }
}

// impl Hash for ChunkCoord {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         state.write_u64(unsafe {std::mem::transmute::<_, u64>(*self)});
//     }
// }
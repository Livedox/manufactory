// use crate::{voxels::{chunk::{CHUNK_BIT_SHIFT, CHUNK_SIZE}, chunks::Chunks}, bytes::AsFromBytes};
// use super::{global_coord::GlobalCoord, local_coord::LocalCoord};

// #[repr(C)]
// #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
// pub struct ChunkCoord {
//     pub x: i32,
//     pub y: i32,
//     pub z: i32
// }

// impl AsFromBytes for ChunkCoord {}

// impl ChunkCoord {
//     #[inline]
//     pub const fn new(x: i32, y: i32, z: i32) -> Self {
//         Self { x, y, z }
//     }

//     #[inline]
//     pub const fn nindex(&self, w: i32, d: i32, ox: i32, oz: i32) -> usize {
//         ((self.y*d + self.z-oz)*w + self.x-ox) as usize
//     }

//     #[inline]
//     pub fn chunk_index(&self, chunks: &Chunks) -> usize {
//         ((self.y*chunks.depth + self.z-chunks.oz())*chunks.width + self.x-chunks.ox()) as usize
//     }

//     #[inline]
//     pub const fn index_without_offset(&self, width: i32, depth: i32) -> usize {
//         ((self.y*depth + self.z)*width + self.x) as usize
//     }

//     #[inline]
//     pub const fn to_global(self, local: LocalCoord) -> GlobalCoord {
//         GlobalCoord::new(
//             self.x * CHUNK_SIZE as i32 + local.x as i32, 
//             self.y * CHUNK_SIZE as i32 + local.y as i32, 
//             self.z * CHUNK_SIZE as i32 + local.z as i32)
//     }
// }

// impl From<(i32, i32, i32)> for ChunkCoord {
//     #[inline]
//     fn from(xyz: (i32, i32, i32)) -> Self {
//         Self::new(xyz.0, xyz.1, xyz.2)
//     }
// }

// impl From<ChunkCoord> for (i32, i32, i32) {
//     #[inline]
//     fn from(coord: ChunkCoord) -> Self {
//         (coord.x, coord.y, coord.z)
//     }
// }

// impl From<ChunkCoord> for [i32; 3] {
//     #[inline]
//     fn from(coord: ChunkCoord) -> Self {
//         unsafe {std::mem::transmute(coord)}
//     }
// }

// impl From<GlobalCoord> for ChunkCoord {
//     #[inline]
//     fn from(coord: GlobalCoord) -> Self {
//         ChunkCoord::new(
//             coord.x >> CHUNK_BIT_SHIFT,
//             coord.y >> CHUNK_BIT_SHIFT,
//             coord.z >> CHUNK_BIT_SHIFT)
//     }
// }
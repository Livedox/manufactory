use crate::{voxels::{chunk::{CHUNK_BIT_SHIFT, CHUNK_SIZE}, chunks::Chunks}, bytes::AsFromBytes};
use super::{global_coords::GlobalCoords, local_coords::LocalCoords};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ChunkCoords(pub i32, pub i32, pub i32);

impl AsFromBytes for ChunkCoords {}

impl ChunkCoords {
    #[inline]
    pub fn nindex(&self, w: i32, d: i32, ox: i32, oz: i32) -> usize {
        ((self.1*d + self.2-oz)*w + self.0-ox) as usize
    }

    #[inline]
    pub fn chunk_index(&self, chunks: &Chunks) -> usize {
        ((self.1*chunks.depth + self.2-chunks.oz())*chunks.width + self.0-chunks.ox()) as usize
    }

    #[inline]
    pub fn index_without_offset(&self, width: i32, depth: i32) -> usize {
        ((self.1*depth + self.2)*width + self.0) as usize
    }

    #[inline]
    pub fn to_global(self, l: LocalCoords) -> GlobalCoords {
        GlobalCoords(
            self.0 * CHUNK_SIZE as i32 + l.0 as i32, 
            self.1 * CHUNK_SIZE as i32 + l.1 as i32, 
            self.2 * CHUNK_SIZE as i32 + l.2 as i32)
    }
}

impl From<(i32, i32, i32)> for ChunkCoords {
    #[inline]
    fn from(xyz: (i32, i32, i32)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<ChunkCoords> for (i32, i32, i32) {
    #[inline]
    fn from(xyz: ChunkCoords) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<ChunkCoords> for [i32; 3] {
    #[inline]
    fn from(xyz: ChunkCoords) -> Self {[xyz.0, xyz.1, xyz.2]}
}

impl From<GlobalCoords> for ChunkCoords {
    #[inline]
    fn from(coords: GlobalCoords) -> Self {
        ChunkCoords(
            coords.0 >> CHUNK_BIT_SHIFT,
            coords.1 >> CHUNK_BIT_SHIFT,
            coords.2 >> CHUNK_BIT_SHIFT)
    }
}
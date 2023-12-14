use crate::{voxels::chunk::{CHUNK_SIZE, CHUNK_BITS}, bytes::AsFromBytes};

use super::{global_coords::GlobalCoords, chunk_coords::ChunkCoords};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LocalCoords(pub u8, pub u8, pub u8);

impl AsFromBytes for LocalCoords {}

impl LocalCoords {
    #[inline]
    pub fn index(&self) -> usize {
        (self.1 as usize*CHUNK_SIZE + self.2 as usize)*CHUNK_SIZE + self.0 as usize
    }

    #[inline]
    pub fn to_global(self, coords: ChunkCoords) -> GlobalCoords {
        GlobalCoords(
            coords.0 * CHUNK_SIZE as i32 + self.0 as i32, 
            coords.1 * CHUNK_SIZE as i32 + self.1 as i32, 
            coords.2 * CHUNK_SIZE as i32 + self.2 as i32)
    }
}

impl From<(u8, u8, u8)> for LocalCoords {
    #[inline] fn from(xyz: (u8, u8, u8)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<(usize, usize, usize)> for LocalCoords {
    #[inline] fn from(xyz: (usize, usize, usize)) -> Self {Self(xyz.0 as u8, xyz.1 as u8, xyz.2 as u8)}
}

impl From<LocalCoords> for (u8, u8, u8) {
    #[inline] fn from(xyz: LocalCoords) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<LocalCoords> for [u8; 3] {
    #[inline] fn from(xyz: LocalCoords) -> Self {[xyz.0, xyz.1, xyz.2]}
}

impl From<LocalCoords> for (usize, usize, usize) {
    #[inline] fn from(xyz: LocalCoords) -> Self {(xyz.0 as usize, xyz.1 as usize, xyz.2 as usize)}
}

impl From<LocalCoords> for [usize; 3] {
    #[inline] fn from(xyz: LocalCoords) -> Self {[xyz.0 as usize, xyz.1 as usize, xyz.2 as usize]}
}

impl From<GlobalCoords> for LocalCoords {
    #[inline]
    fn from(coords: GlobalCoords) -> Self {
        let lx = coords.0 & CHUNK_BITS as i32;
        let ly = coords.1 & CHUNK_BITS as i32;
        let lz = coords.2 & CHUNK_BITS as i32;
        LocalCoords(lx as u8, ly as u8, lz as u8)
    }
}
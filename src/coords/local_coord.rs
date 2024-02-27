use crate::{voxels::chunk::{CHUNK_SIZE, CHUNK_BITS}, bytes::AsFromBytes};

use super::{chunk_coord::ChunkCoord, global_coord::GlobalCoord};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LocalCoord {
    pub x: u8,
    pub y: u8,
    pub z: u8
}

impl AsFromBytes for LocalCoord {}

impl LocalCoord {
    #[inline] pub const fn new(x: u8, y: u8, z: u8) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn index(&self) -> usize {
        (self.y as usize*CHUNK_SIZE + self.z as usize)*CHUNK_SIZE + self.x as usize
    }

    #[inline]
    pub const fn to_global(self, coord: ChunkCoord) -> GlobalCoord {
        GlobalCoord::new(
            coord.x * CHUNK_SIZE as i32 + self.x as i32, 
            coord.y * CHUNK_SIZE as i32 + self.y as i32, 
            coord.z * CHUNK_SIZE as i32 + self.z as i32)
    }
}

impl From<(u8, u8, u8)> for LocalCoord {
    #[inline] fn from(xyz: (u8, u8, u8)) -> Self {
        unsafe {std::mem::transmute(xyz)}
    }
}

impl From<(usize, usize, usize)> for LocalCoord {
    #[inline] fn from(xyz: (usize, usize, usize)) -> Self {
        Self::new(xyz.0 as u8, xyz.1 as u8, xyz.2 as u8)
    }
}

impl From<LocalCoord> for (u8, u8, u8) {
    #[inline] fn from(coord: LocalCoord) -> Self {
        unsafe {std::mem::transmute(coord)}
    }
}

impl From<LocalCoord> for [u8; 3] {
    #[inline] fn from(coord: LocalCoord) -> Self {
        unsafe {std::mem::transmute(coord)}
    }
}

impl From<LocalCoord> for (usize, usize, usize) {
    #[inline] fn from(coord: LocalCoord) -> Self {(coord.x as usize, coord.y as usize, coord.z as usize)}
}

impl From<LocalCoord> for [usize; 3] {
    #[inline] fn from(coord: LocalCoord) -> Self {[coord.x as usize, coord.y as usize, coord.z as usize]}
}

impl From<GlobalCoord> for LocalCoord {
    #[inline]
    fn from(coord: GlobalCoord) -> Self {
        let lx = coord.x & CHUNK_BITS as i32;
        let ly = coord.y & CHUNK_BITS as i32;
        let lz = coord.z & CHUNK_BITS as i32;
        LocalCoord::new(lx as u8, ly as u8, lz as u8)
    }
}
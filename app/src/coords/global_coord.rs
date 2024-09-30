use std::ops::{Add, AddAssign, Sub};

use serde::{Deserialize, Serialize};

use crate::voxels::chunk::{CHUNK_BITS, CHUNK_BIT_SHIFT};

use super::{chunk_coord::ChunkCoord, local_coord::LocalCoord};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GlobalCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GlobalCoord {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {Self { x, y, z }}
}

impl From<(i32, i32, i32)> for GlobalCoord {
    #[inline]
    fn from(v: (i32, i32, i32)) -> Self {
        Self { x: v.0, y: v.1, z: v.2 }
    }
}

impl From<(f32, f32, f32)> for GlobalCoord {
    #[inline]
    fn from(v: (f32, f32, f32)) -> Self {
        Self { x: v.0 as i32, y: v.1 as i32, z: v.2 as i32 }
    }
}

impl From<GlobalCoord> for (i32, i32, i32) {
    #[inline]
    fn from(v: GlobalCoord) -> Self {
        (v.x, v.y, v.z)
    }
}

impl From<GlobalCoord> for [i32; 3] {
    #[inline]
    fn from(v: GlobalCoord) -> Self {
        [v.x, v.y, v.z]
    }
}

impl From<GlobalCoord> for [f32; 3] {
    #[inline]
    fn from(v: GlobalCoord) -> Self {
        [v.x as f32, v.y as f32, v.z as f32]
    }
}

impl From<GlobalCoord> for (f32, f32, f32) {
    #[inline]
    fn from(v: GlobalCoord) -> Self {
        (v.x as f32, v.y as f32, v.z as f32)
    }
}

impl From<GlobalCoord> for ChunkCoord {
    #[inline]
    fn from(coord: GlobalCoord) -> Self {
        ChunkCoord::new(
            coord.x >> CHUNK_BIT_SHIFT,
            coord.z >> CHUNK_BIT_SHIFT)
    }
}

impl From<GlobalCoord> for LocalCoord {
    #[inline]
    fn from(coord: GlobalCoord) -> Self {
        let lx = coord.x & CHUNK_BITS as i32;
        let lz = coord.z & CHUNK_BITS as i32;
        LocalCoord::new(lx as usize, coord.y as usize, lz as usize)
    }
}

impl Add for GlobalCoord {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for GlobalCoord {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl AddAssign<&GlobalCoord> for GlobalCoord {
    #[inline]
    fn add_assign(&mut self, rhs: &GlobalCoord) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for GlobalCoord {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
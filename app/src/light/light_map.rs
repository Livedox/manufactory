use std::{ops::Index, sync::atomic::{AtomicU8, Ordering}};

use crate::{coords::local_coord::LocalCoord, voxels::chunk::CHUNK_VOLUME};
use crate::light::light::Light;

#[derive(Debug)]
pub struct LightMap(pub [Light; CHUNK_VOLUME]);

impl LightMap {
    #[inline]
    pub fn new() -> Self {Self::default()}

    #[inline]
    pub fn get(&self, lc: LocalCoord) -> Option<&Light> { 
        self.0.get(lc.index())
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, lc: LocalCoord) -> &Light { 
        unsafe {self.0.get_unchecked(lc.index())}
    }
}

impl Default for LightMap {
    #[inline]
    fn default() -> Self {Self(unsafe {std::mem::zeroed()})}
}

impl Index<LocalCoord> for LightMap {
    type Output = Light;
    fn index(&self, index: LocalCoord) -> &Self::Output {
        &self.0[index.index()]
    }
}
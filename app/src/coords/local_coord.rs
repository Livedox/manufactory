use crate::voxels::chunk::{CHUNK_BITS, CHUNK_BIT_SHIFT, CHUNK_SIZE};

#[derive(Debug, Clone, Copy)]
pub struct LocalCoord {pub x: usize, pub y: usize, pub z: usize}
impl LocalCoord {
    #[inline]
    pub fn new(x: usize, y: usize, z: usize) -> Self {Self { x, y, z }}

    #[inline]
    pub fn index(&self) -> usize {
        (self.y * CHUNK_SIZE + self.z)*CHUNK_SIZE + self.x
    }

    pub fn from_index(mut index: usize) -> Self {
        let x = index & CHUNK_BITS;
        index >>= CHUNK_BIT_SHIFT;
        let z = index & CHUNK_BITS;
        index >>= CHUNK_BIT_SHIFT;
        Self {x, z, y: index}
    }
}

impl From<(usize, usize, usize)> for LocalCoord {
    fn from(value: (usize, usize, usize)) -> Self {
        Self { x: value.0, y: value.1, z: value.2 }
    }
}
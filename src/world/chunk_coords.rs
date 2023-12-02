use crate::{voxels::{chunk::{CHUNK_BIT_SHIFT, CHUNK_SIZE}, chunks::Chunks}, bytes::{DynByteInterpretation, ConstByteInterpretation}};
use crate::bytes::NumFromBytes;
use super::{global_coords::GlobalCoords, local_coords::LocalCoords};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub struct ChunkCoords(pub i32, pub i32, pub i32);

impl ChunkCoords {
    #[inline]
    pub fn nindex(&self, w: i32, d: i32, ox: i32, oz: i32) -> usize {
        ((self.1*d + self.2-oz)*w + self.0-ox) as usize
    }

    #[inline]
    pub fn chunk_index(&self, chunks: &Chunks) -> usize {
        ((self.1*chunks.depth + self.2-chunks.oz)*chunks.width + self.0-chunks.ox) as usize
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


impl DynByteInterpretation for ChunkCoords {
    fn from_bytes(data: &[u8]) -> Self {
        ChunkCoords(i32::from_bytes(&data[0..4]),
            i32::from_bytes(&data[4..8]),
            i32::from_bytes(&data[8..12]))
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut v = Vec::with_capacity(12);
        v.extend(self.0.to_le_bytes());
        v.extend(self.1.to_le_bytes());
        v.extend(self.2.to_le_bytes());
        v.into()
    }
}


#[cfg(test)]
mod tests {
    use crate::{world::{chunk_coords::ChunkCoords, global_coords::GlobalCoords}, voxels::chunk::CHUNK_SIZE};


    #[test]
    fn correct_from_global_coords() {
        let g0 = GlobalCoords(18, 0, 134);
        let g1 = GlobalCoords(-1, -18, -196);

        let c0 = ChunkCoords(
            g0.0 / CHUNK_SIZE as i32,
            g0.1 / CHUNK_SIZE as i32,
            g0.2 / CHUNK_SIZE as i32);

        let c1 = ChunkCoords(
            g1.0 / CHUNK_SIZE as i32 - 1,
            g1.1 / CHUNK_SIZE as i32 - 1,
            g1.2 / CHUNK_SIZE as i32 - 1);

        assert_eq!(c0, g0.into());
        assert_eq!(c1, g1.into());
    }
}
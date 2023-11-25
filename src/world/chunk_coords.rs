use crate::voxels::{chunk::{CHUNK_BIT_SHIFT, CHUNK_SIZE}, chunks::Chunks};

use super::{global_coords::GlobalCoords, local_coords::LocalCoords};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ChunkCoords(pub i32, pub i32, pub i32);

impl ChunkCoords {
    pub fn nindex(&self, w: i32, d: i32, ox: i32, oz: i32) -> usize {
        ((self.1*d + self.2-oz)*w + self.0-ox) as usize
    }

    pub fn chunk_index(&self, chunks: &Chunks) -> usize {
        ((self.1*chunks.depth + self.2-chunks.oz)*chunks.width + self.0-chunks.ox) as usize
    }

    pub fn index_without_offset(&self, width: i32, depth: i32) -> usize {
        ((self.1*depth + self.2)*width + self.0) as usize
    }

    pub fn to_global(self, local: LocalCoords) -> GlobalCoords {
        let mut lx = local.0 as i8;
        let mut ly = local.1 as i8;
        let mut lz = local.2 as i8;
        let mut cx = self.0;
        let mut cy = self.1;
        let mut cz = self.2;
        // if self.0 < 0 {
        //     lx = -(CHUNK_SIZE as i8 - lx - 1);
        //     cx += 1;
        // }
        // if self.1 < 0 {
        //     ly = -(CHUNK_SIZE as i8 - ly - 1);
        //     cy += 1;
        // }
        // if self.2 < 0 {
        //     lz = -(CHUNK_SIZE as i8 - lz - 1);
        //     cz += 1;
        // }
        GlobalCoords(
            cx * CHUNK_SIZE as i32 + lx as i32, 
            cy * CHUNK_SIZE as i32 + ly as i32, 
            cz * CHUNK_SIZE as i32 + lz as i32)
    }
}

impl From<(i32, i32, i32)> for ChunkCoords {
    fn from(xyz: (i32, i32, i32)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<ChunkCoords> for (i32, i32, i32) {
    fn from(xyz: ChunkCoords) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<ChunkCoords> for [i32; 3] {
    fn from(xyz: ChunkCoords) -> Self {[xyz.0, xyz.1, xyz.2]}
}

impl From<GlobalCoords> for ChunkCoords {
    fn from(coords: GlobalCoords) -> Self {
        ChunkCoords(
            coords.0 >> CHUNK_BIT_SHIFT,
            coords.1 >> CHUNK_BIT_SHIFT,
            coords.2 >> CHUNK_BIT_SHIFT
        )
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
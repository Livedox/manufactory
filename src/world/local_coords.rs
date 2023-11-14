use crate::voxels::chunk::CHUNK_SIZE;

use super::{global_coords::GlobalCoords, chunk_coords::ChunkCoords};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LocalCoords(pub u8, pub u8, pub u8);

impl LocalCoords {
    pub fn index(&self) -> usize {
        (self.1 as usize*CHUNK_SIZE + self.2 as usize)*CHUNK_SIZE + self.0 as usize
    }

    pub fn to_global(self, coords: ChunkCoords) -> GlobalCoords {
        GlobalCoords(
            coords.0 * CHUNK_SIZE as i32 + self.0 as i32, 
            coords.1 * CHUNK_SIZE as i32 + self.1 as i32, 
            coords.2 * CHUNK_SIZE as i32 + self.2 as i32)
    }
}

impl From<(u8, u8, u8)> for LocalCoords {
    fn from(xyz: (u8, u8, u8)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<(usize, usize, usize)> for LocalCoords {
    fn from(xyz: (usize, usize, usize)) -> Self {Self(xyz.0 as u8, xyz.1 as u8, xyz.2 as u8)}
}

impl From<LocalCoords> for (u8, u8, u8) {
    fn from(xyz: LocalCoords) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<LocalCoords> for [u8; 3] {
    fn from(xyz: LocalCoords) -> Self {[xyz.0, xyz.1, xyz.2]}
}

impl From<LocalCoords> for (usize, usize, usize) {
    fn from(xyz: LocalCoords) -> Self {(xyz.0 as usize, xyz.1 as usize, xyz.2 as usize)}
}

impl From<LocalCoords> for [usize; 3] {
    fn from(xyz: LocalCoords) -> Self {[xyz.0 as usize, xyz.1 as usize, xyz.2 as usize]}
}

impl From<GlobalCoords> for LocalCoords {
    fn from(coords: GlobalCoords) -> Self {
        LocalCoords(
            (coords.0.unsigned_abs() % CHUNK_SIZE as u32) as u8,
            (coords.1.unsigned_abs() % CHUNK_SIZE as u32) as u8,
            (coords.2.unsigned_abs() % CHUNK_SIZE as u32) as u8
        )
    }
}


#[cfg(test)]
mod tests {
    use crate::{world::{global_coords::GlobalCoords, local_coords::LocalCoords}, voxels::chunk::CHUNK_SIZE};
    #[test]
    fn correct_from_global_coords() {
        let g0 = GlobalCoords(18, 0, 134);
        let g1 = GlobalCoords(-1, -18, -196);

        let l0 = LocalCoords(
            (g0.0.unsigned_abs() % CHUNK_SIZE as u32) as u8,
            (g0.1.unsigned_abs() % CHUNK_SIZE as u32) as u8,
            (g0.2.unsigned_abs() % CHUNK_SIZE as u32) as u8);

        let l1 = LocalCoords(
            (g1.0.unsigned_abs() % CHUNK_SIZE as u32) as u8,
            (g1.1.unsigned_abs() % CHUNK_SIZE as u32) as u8,
            (g1.2.unsigned_abs() % CHUNK_SIZE as u32) as u8);

        assert_eq!(l0, g0.into());
        assert_eq!(l1, g1.into());
    }
}
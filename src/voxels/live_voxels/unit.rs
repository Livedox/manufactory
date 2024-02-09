use super::{LiveVoxel, LiveVoxelDesiarialize, LiveVoxelNew, PlayerUnlockable};

impl LiveVoxel for () {
    fn serialize(&self) -> Vec<u8> {vec![]}
}
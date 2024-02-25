use super::LiveVoxelBehavior;

impl LiveVoxelBehavior for () {
    fn serialize(&self) -> Vec<u8> {vec![]}
}
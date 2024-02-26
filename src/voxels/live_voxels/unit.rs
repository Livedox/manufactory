use super::LiveVoxelBehavior;

impl LiveVoxelBehavior for () {
    fn to_bytes(&self) -> Vec<u8> {vec![]}
}
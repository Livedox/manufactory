use crate::bytes::ConstByteInterpretation;
use crate::bytes::NumFromBytes;

#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub id: u32,
}

impl Voxel {
    pub fn new(id: u32) -> Voxel {
        Voxel { id }
    }
}


impl ConstByteInterpretation for Voxel {
    fn to_bytes(&self) -> Box<[u8]> {
        Box::new(self.id.to_le_bytes())
    }

    fn from_bytes(data: &[u8]) -> Self {
        Self { id: u32::from_bytes(&data[0..4]) }
    }

    fn size(&self) -> u32 {4}
}
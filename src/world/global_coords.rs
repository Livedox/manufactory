use crate::bytes::ConstByteInterpretation;
use crate::bytes::NumFromBytes;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GlobalCoords(pub i32, pub i32, pub i32);

impl From<(i32, i32, i32)> for GlobalCoords {
    fn from(xyz: (i32, i32, i32)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<GlobalCoords> for (i32, i32, i32) {
    fn from(xyz: GlobalCoords) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<(f32, f32, f32)> for GlobalCoords {
    fn from(xyz: (f32, f32, f32)) -> Self {Self(xyz.0 as i32, xyz.1 as i32, xyz.2 as i32)}
}

impl From<GlobalCoords> for (f32, f32, f32) {
    fn from(xyz: GlobalCoords) -> Self {(xyz.0 as f32, xyz.1 as f32, xyz.2 as f32)}
}

impl From<GlobalCoords> for [i32; 3] {
    fn from(xyz: GlobalCoords) -> Self {[xyz.0, xyz.1, xyz.2]}
}

impl ConstByteInterpretation for GlobalCoords {
    fn from_bytes(data: &[u8]) -> Self {
        GlobalCoords(i32::from_bytes(&data[0..4]),
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

    fn size(&self) -> u32 {12}
}
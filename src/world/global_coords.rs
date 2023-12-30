use crate::bytes::AsFromBytes;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GlobalCoords(pub i32, pub i32, pub i32);

impl AsFromBytes for GlobalCoords {}

impl From<(i32, i32, i32)> for GlobalCoords {
    #[inline]
    fn from(xyz: (i32, i32, i32)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<GlobalCoords> for (i32, i32, i32) {
    #[inline]
    fn from(xyz: GlobalCoords) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<(f32, f32, f32)> for GlobalCoords {
    #[inline]
    fn from(xyz: (f32, f32, f32)) -> Self {Self(xyz.0 as i32, xyz.1 as i32, xyz.2 as i32)}
}

impl From<GlobalCoords> for (f32, f32, f32) {
    #[inline]
    fn from(xyz: GlobalCoords) -> Self {(xyz.0 as f32, xyz.1 as f32, xyz.2 as f32)}
}

impl From<GlobalCoords> for [f32; 3] {
    #[inline]
    fn from(xyz: GlobalCoords) -> Self {[xyz.0 as f32, xyz.1 as f32, xyz.2 as f32]}
}

impl From<GlobalCoords> for [i32; 3] {
    #[inline]
    fn from(xyz: GlobalCoords) -> Self {[xyz.0, xyz.1, xyz.2]}
}
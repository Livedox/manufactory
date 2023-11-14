#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GlobalCoords(pub i32, pub i32, pub i32);

impl From<(i32, i32, i32)> for GlobalCoords {
    fn from(xyz: (i32, i32, i32)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<GlobalCoords> for (i32, i32, i32) {
    fn from(xyz: GlobalCoords) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<GlobalCoords> for [i32; 3] {
    fn from(xyz: GlobalCoords) -> Self {[xyz.0, xyz.1, xyz.2]}
}
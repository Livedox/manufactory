#[derive(Debug, Clone, Copy)]
pub struct GlobalXYZ(pub i32, pub i32, pub i32);

impl From<(i32, i32, i32)> for GlobalXYZ {
    fn from(xyz: (i32, i32, i32)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<GlobalXYZ> for (i32, i32, i32) {
    fn from(xyz: GlobalXYZ) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<GlobalXYZ> for [i32; 3] {
    fn from(xyz: GlobalXYZ) -> Self {[xyz.0, xyz.1, xyz.2]}
}
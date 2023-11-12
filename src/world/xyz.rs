#[derive(Debug, Clone, Copy)]
pub struct XYZ(pub f32, pub f32, pub f32);

impl From<(f32, f32, f32)> for XYZ {
    fn from(xyz: (f32, f32, f32)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<XYZ> for (f32, f32, f32) {
    fn from(xyz: XYZ) -> Self {(xyz.0, xyz.1, xyz.2)}
}


impl From<XYZ> for [f32; 3] {
    fn from(xyz: XYZ) -> Self {[xyz.0, xyz.1, xyz.2]}
}

impl From<[f32; 3]> for XYZ {
    fn from(xyz: [f32; 3]) -> Self {XYZ(xyz[0], xyz[1], xyz[2])}
}
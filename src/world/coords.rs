#[derive(Debug, Clone, Copy)]
pub struct Coords(pub f32, pub f32, pub f32);

impl From<(f32, f32, f32)> for Coords {
    fn from(xyz: (f32, f32, f32)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<Coords> for (f32, f32, f32) {
    fn from(xyz: Coords) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<Coords> for [f32; 3] {
    fn from(xyz: Coords) -> Self {[xyz.0, xyz.1, xyz.2]}
}

impl From<[f32; 3]> for Coords {
    fn from(xyz: [f32; 3]) -> Self {Coords(xyz[0], xyz[1], xyz[2])}
}
use std::ops::Add;

use super::global_coords::GlobalCoords;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Coords(pub f32, pub f32, pub f32);

impl From<(f32, f32, f32)> for Coords {
    #[inline]
    fn from(xyz: (f32, f32, f32)) -> Self {Self(xyz.0, xyz.1, xyz.2)}
}

impl From<Coords> for (f32, f32, f32) {
    #[inline]
    fn from(xyz: Coords) -> Self {(xyz.0, xyz.1, xyz.2)}
}

impl From<Coords> for [f32; 3] {
    #[inline]
    fn from(xyz: Coords) -> Self {[xyz.0, xyz.1, xyz.2]}
}

impl From<[f32; 3]> for Coords {
    #[inline]
    fn from(xyz: [f32; 3]) -> Self {Coords(xyz[0], xyz[1], xyz[2])}
}

impl From<GlobalCoords> for Coords {
    #[inline]
    fn from(value: GlobalCoords) -> Self {
        Self(value.0 as f32, value.1 as f32, value.2 as f32)
    }
}

impl Add for Coords {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}
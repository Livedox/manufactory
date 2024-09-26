use std::ops::Add;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Coord {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Coord {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl From<(f32, f32, f32)> for Coord {
    #[inline]
    fn from(xyz: (f32, f32, f32)) -> Self {
        unsafe {std::mem::transmute(xyz)}
    }
}

impl From<Coord> for (f32, f32, f32) {
    #[inline]
    fn from(coord: Coord) -> Self {
        unsafe {std::mem::transmute(coord)}
    }
}

impl From<Coord> for [f32; 3] {
    #[inline]
    fn from(coord: Coord) -> Self {
        unsafe {std::mem::transmute(coord)}
    }
}

impl From<[f32; 3]> for Coord {
    #[inline]
    fn from(xyz: [f32; 3]) -> Self {
        unsafe {std::mem::transmute(xyz)}
    }
}

// impl From<GlobalCoord> for Coord {
//     #[inline]
//     fn from(coord: GlobalCoord) -> Self {
//         Self::new(coord.x as f32, coord.y as f32, coord.z as f32)
//     }
// }

impl Add for Coord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
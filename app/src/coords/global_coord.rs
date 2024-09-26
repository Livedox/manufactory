// use std::ops::{Add, Sub};
// use std::ops::AddAssign;
// use serde::{Deserialize, Serialize};

// use crate::bytes::AsFromBytes;

// #[repr(C)]
// #[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
// pub struct GlobalCoord {
//     pub x: i32,
//     pub y: i32,
//     pub z: i32
// }

// impl GlobalCoord {
//     #[inline]
//     pub const fn new(x: i32, y: i32, z: i32) -> Self {
//         Self { x, y, z }
//     }
// }

// impl AsFromBytes for GlobalCoord {}

// impl From<(i32, i32, i32)> for GlobalCoord {
//     #[inline]
//     fn from(xyz: (i32, i32, i32)) -> Self {
//         Self::new(xyz.0, xyz.1, xyz.2)
//     }
// }

// impl From<GlobalCoord> for (i32, i32, i32) {
//     #[inline]
//     fn from(coord: GlobalCoord) -> Self {
//         (coord.x, coord.y, coord.z)
//     }
// }

// impl From<GlobalCoord> for [i32; 3] {
//     #[inline]
//     fn from(coord: GlobalCoord) -> Self {
//         unsafe {std::mem::transmute(coord)}
//     }
// }

// impl From<(f32, f32, f32)> for GlobalCoord {
//     #[inline]
//     fn from(xyz: (f32, f32, f32)) -> Self {Self {
//         x: xyz.0 as i32, y: xyz.1 as i32, z: xyz.2 as i32}}
// }

// impl From<GlobalCoord> for (f32, f32, f32) {
//     #[inline]
//     fn from(coord: GlobalCoord) -> Self {(coord.x as f32, coord.y as f32, coord.z as f32)}
// }

// impl From<GlobalCoord> for [f32; 3] {
//     #[inline]
//     fn from(coord: GlobalCoord) -> Self {[coord.x as f32, coord.y as f32, coord.z as f32]}
// }

// impl Add for GlobalCoord {
//     type Output = Self;
//     #[inline(always)]
//     fn add(self, rhs: Self) -> Self::Output {
//         Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
//     }
// }

// impl AddAssign for GlobalCoord {
//     #[inline]
//     fn add_assign(&mut self, rhs: Self) {
//         self.x += rhs.x;
//         self.y += rhs.y;
//         self.z += rhs.z;
//     }
// }

// impl AddAssign<&GlobalCoord> for GlobalCoord {
//     #[inline]
//     fn add_assign(&mut self, rhs: &GlobalCoord) {
//         self.x += rhs.x;
//         self.y += rhs.y;
//         self.z += rhs.z;
//     }
// }

// impl Sub for GlobalCoord {
//     type Output = Self;
//     #[inline]
//     fn sub(self, rhs: Self) -> Self::Output {
//         Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
//     }
// }
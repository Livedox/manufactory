use std::sync::atomic::{AtomicU16, Ordering};

use crate::voxels::chunk::{CHUNK_VOLUME, CHUNK_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Light(pub u16);

impl Default for Light {
    fn default() -> Self {Self(0x0000)}
}

impl Light {
    const MAX_VALUE: u8 = 15;
    #[inline] pub fn new(light: u16) -> Self {Self(light)}

    #[inline] pub fn get(self, channel: u8) -> u16 {(self.0 >> (channel << 2)) & 0xF}
    #[inline] pub fn get_red(self) -> u16 {self.0 & 0xF}
    #[inline] pub fn get_green(self) -> u16 {(self.0 >> 4) & 0xF}
    #[inline] pub fn get_blue(self) -> u16 {(self.0 >> 8) & 0xF}
    #[inline] pub fn get_sun(self) -> u16 {(self.0 >> 12) & 0xF}

    #[inline]
    pub fn set(&mut self, value: u16, channel: u8) {
        self.0 = (self.0 & (!(0xF << (channel*4)))) | (value << (channel << 2));
    }
    #[inline] pub fn set_red(&mut self, value: u16) {self.0 = self.0 & 0xFFF0 | value}
    #[inline] pub fn set_green(&mut self, value: u16) {self.0 = self.0 & 0xFFF0 | (value << 4)}
    #[inline] pub fn set_blue(&mut self, value: u16) {self.0 = self.0 & 0xFFF0 | (value << 8)}
    #[inline] pub fn set_sun(&mut self, value: u16) {self.0 = self.0 & 0xFFF0 | (value << 12)}

    pub fn get_normalized(&self) -> [f32; 4] {
        [self.get_red() as f32 / Self::MAX_VALUE as f32,
         self.get_green() as f32 / Self::MAX_VALUE as f32,
         self.get_blue() as f32 / Self::MAX_VALUE as f32,
         self.get_sun() as f32 / Self::MAX_VALUE as f32]
    }
}

#[derive(Debug)]
pub struct LightMap {
    pub map: [LightAtomic; CHUNK_VOLUME],
}

impl LightMap {
    #[inline]
    pub fn new() -> LightMap {
        LightMap { map: unsafe {std::mem::zeroed()}}
    }

    #[inline]
    fn index(local: (u8, u8, u8)) -> usize {
        let (x, y, z) = (local.0 as u16, local.1 as u16, local.2 as u16);
        ((y * CHUNK_SIZE as u16 + z) * CHUNK_SIZE as u16 + x) as usize
    }

    #[inline]
    pub fn get_light(&self, local: (u8, u8, u8)) -> Light {
        self.map[LightMap::index(local)].to_light()
    }

    #[inline]
    pub fn get(&self, local: (u8, u8, u8), channel: u8) -> u16 { 
        self.map[LightMap::index(local)].get(channel)
    }

    #[inline]
    pub fn get_red(&self, local: (u8, u8, u8)) -> u16 {
        self.map[LightMap::index(local)].get_red()
    }

    #[inline]
    pub fn get_green(&self, local: (u8, u8, u8)) -> u16 {
        self.map[LightMap::index(local)].get_green()
    }

    #[inline]
    pub fn get_blue(&self, local: (u8, u8, u8)) -> u16 {
        self.map[LightMap::index(local)].get_blue()
    }

    #[inline]
    pub fn get_sun(&self, local: (u8, u8, u8)) -> u16 {
        self.map[LightMap::index(local)].get_sun()
    }

    #[inline]
    pub fn set_red(&self, local: (u8, u8, u8), value: u16) {
        self.map[LightMap::index(local)].set_red(value);
    }

    #[inline]
    pub fn set_green(&self, local: (u8, u8, u8), value: u16) {
        self.map[LightMap::index(local)].set_green(value);
    }

    #[inline]
    pub fn set_blue(&self, local: (u8, u8, u8), value: u16) {
        self.map[LightMap::index(local)].set_blue(value);
    }

    #[inline]
    pub fn set_sun(&self, local: (u8, u8, u8), value: u16) {
        self.map[LightMap::index(local)].set_sun(value);
    }

    #[inline]
    pub fn set(&self, local: (u8, u8, u8), value: u16, channel: u8) {
        self.map[LightMap::index(local)].set(value, channel);
    }
}

impl Default for LightMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct LightAtomic(pub AtomicU16);

impl LightAtomic {
    const MAX_VALUE: u8 = 15;

    #[inline]
    pub fn to_light(&self) -> Light {
        Light(self.0.load(Ordering::Relaxed))
    }
    
    #[inline]
    pub fn new(light: u16) -> Self {Self(AtomicU16::new(light))}
    #[inline] pub fn get(&self, channel: u8) -> u16 {(self.0.load(Ordering::Relaxed) >> (channel << 2)) & 0xF}
    #[inline] pub fn get_red(&self) -> u16 {self.0.load(Ordering::Relaxed) & 0xF}
    #[inline] pub fn get_green(&self) -> u16 {(self.0.load(Ordering::Relaxed) >> 4) & 0xF}
    #[inline] pub fn get_blue(&self) -> u16 {(self.0.load(Ordering::Relaxed) >> 8) & 0xF}
    #[inline] pub fn get_sun(&self) -> u16 {(self.0.load(Ordering::Relaxed) >> 12) & 0xF}

    #[inline]
    pub fn set(&self, value: u16, channel: u8) {
        let _ = self.0.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |l| {
            Some((l & (!(0xF << (channel*4)))) | (value << (channel << 2)))
        });
    }

    #[inline]
    pub fn set_red(&self, value: u16) {
        let _ = self.0.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |l| Some(l & 0xFFF0 | value));
    }
    #[inline]
    pub fn set_green(&self, value: u16) {
        let _ = self.0.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |l| Some(l & 0xFFF0 | (value << 4)));
    }
    #[inline]
    pub fn set_blue(&self, value: u16) {
        let _ = self.0.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |l| Some(l & 0xFFF0 | (value << 8)));
    }
    #[inline]
    pub fn set_sun(&self, value: u16) {
        let _ = self.0.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |l| Some(l & 0xFFF0 | (value << 12)));
    }

    pub fn get_normalized(&self) -> [f32; 4] {
        [self.get_red() as f32 / Self::MAX_VALUE as f32,
         self.get_green() as f32 / Self::MAX_VALUE as f32,
         self.get_blue() as f32 / Self::MAX_VALUE as f32,
         self.get_sun() as f32 / Self::MAX_VALUE as f32]
    }
}

impl Default for LightAtomic {
    fn default() -> Self {
        Self(AtomicU16::new(0))
    }
}

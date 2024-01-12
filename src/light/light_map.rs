use std::{sync::atomic::{AtomicU16, Ordering, AtomicU8}, cell::UnsafeCell};

use crate::{voxels::chunk::{CHUNK_VOLUME, CHUNK_SIZE}, world::local_coords::LocalCoords};

#[derive(Debug)]
pub struct Light(pub [AtomicU8; 4]);
impl Default for Light {
    #[inline]
    fn default() -> Self {Self(unsafe {std::mem::zeroed()})}
}
impl Clone for Light {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {std::mem::transmute::<_, Self>(*self.0.as_ptr().cast::<u32>())}
    }
}
impl PartialEq for Light {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.to_number() == other.to_number()
    }
}
impl Eq for Light {}

impl Light {
    const MAX_VALUE: u8 = 15;
    #[inline] pub fn new(r: u8, g: u8, b: u8, s: u8) -> Self {
        Self([AtomicU8::new(r), AtomicU8::new(g), AtomicU8::new(b), AtomicU8::new(s)])
    }

    #[inline(always)] pub unsafe fn get_unchecked_channel(&self, channel: usize) -> u8 {
        unsafe {self.0.get_unchecked(channel)}.load(Ordering::Relaxed)
    }
    #[inline] pub fn get_channel(&self, channel: usize) -> u8 {self.0[channel].load(Ordering::Relaxed)}
    #[inline] pub fn get_red(&self) -> u8   {unsafe {self.0.get_unchecked(0)}.load(Ordering::Relaxed)}
    #[inline] pub fn get_green(&self) -> u8 {unsafe {self.0.get_unchecked(1)}.load(Ordering::Relaxed)}
    #[inline] pub fn get_blue(&self) -> u8  {unsafe {self.0.get_unchecked(2)}.load(Ordering::Relaxed)}
    #[inline] pub fn get_sun(&self) -> u8   {unsafe {self.0.get_unchecked(3)}.load(Ordering::Relaxed)}


    #[inline] pub unsafe fn set_unchecked_channel(&self, value: u8, channel: usize) {
        unsafe {self.0.get_unchecked(channel)}.store(value, Ordering::Relaxed);
    }
    #[inline]
    pub fn set(&self, value: u8, channel: usize) {
        self.0[channel].store(value, Ordering::Relaxed);
    }
    #[inline] pub fn set_red(&self, value: u8) {
        unsafe {self.0.get_unchecked(0)}.store(value, Ordering::Relaxed);
    }
    #[inline] pub fn set_green(&self, value: u8) {
        unsafe {self.0.get_unchecked(1)}.store(value, Ordering::Relaxed);
    }
    #[inline] pub fn set_blue(&self, value: u8) {
        unsafe {self.0.get_unchecked(2)}.store(value, Ordering::Relaxed);
    }
    #[inline] pub fn set_sun(&self, value: u8) {
        unsafe {self.0.get_unchecked(3)}.store(value, Ordering::Relaxed);
    }

    pub fn get_normalized(&self) -> [f32; 4] {
        [self.get_red() as f32 / Self::MAX_VALUE as f32,
         self.get_green() as f32 / Self::MAX_VALUE as f32,
         self.get_blue() as f32 / Self::MAX_VALUE as f32,
         self.get_sun() as f32 / Self::MAX_VALUE as f32]
    }

    #[inline(always)]
    pub fn to_number(&self) -> u32 {
        unsafe {*self.0.as_ptr().cast::<u32>()}
    }
}


#[derive(Debug)]
pub struct LightMap {
    pub map: [Light; CHUNK_VOLUME],
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
        self.map[LightMap::index(local)].clone()
    }

    #[inline]
    pub fn get(&self, local: (u8, u8, u8)) -> &Light { 
        &self.map[LightMap::index(local)]
    }

    #[inline]
    pub fn get_unchecked(&self, local: LocalCoords) -> &Light { 
        unsafe {self.map.get_unchecked(local.index())}
    }

    #[inline]
    pub unsafe fn get_unchecked_channel(&self, local: (u8, u8, u8), channel: usize) -> u8 { 
        unsafe {self.map[LightMap::index(local)].get_unchecked_channel(channel)}
    }
}

impl Default for LightMap {
    fn default() -> Self {Self::new()}
}

use crate::voxels::chunk::{CHUNK_VOLUME, CHUNK_SIZE};

#[derive(Debug)]
pub struct LightMap {
    pub map: [u16; CHUNK_VOLUME],
}

impl LightMap {
    pub fn new() -> LightMap {
        LightMap { map: [0x0000; CHUNK_VOLUME]}
    }
    
    fn get_index(local: (u8, u8, u8)) -> usize {
        let (x, y, z) = (local.0 as u16, local.1 as u16, local.2 as u16);
        ((y * CHUNK_SIZE as u16 + z) * CHUNK_SIZE as u16 + x) as usize
    }

    pub fn get_light(&self, local: (u8, u8, u8)) -> u16 {
        self.map[LightMap::get_index(local)]
    }

    pub fn get(&self, local: (u8, u8, u8), channel: u8) -> u16 { 
        (self.map[LightMap::get_index(local)] >> (channel << 2)) & 0xF
    }

    pub fn get_red(&self, local: (u8, u8, u8)) -> u16 {
        self.map[LightMap::get_index(local)] & 0xF
    }

    pub fn get_green(&self, local: (u8, u8, u8)) -> u16 {
        (self.map[LightMap::get_index(local)] >> 4) & 0xF
    }

    pub fn get_blue(&self, local: (u8, u8, u8)) -> u16 {
        (self.map[LightMap::get_index(local)] >> 8) & 0xF
    }

    pub fn get_sun(&self, local: (u8, u8, u8)) -> u16 {
        (self.map[LightMap::get_index(local)] >> 12) & 0xF
    }

    pub fn set_red(&mut self, local: (u8, u8, u8), value: u16) {
        let index = LightMap::get_index(local);
        self.map[index] = self.map[index] & 0xFFF0 | value;
    }

    pub fn set_green(&mut self, local: (u8, u8, u8), value: u16) {
        let index = LightMap::get_index(local);
        self.map[index] = self.map[index] & 0xFFF0 | (value << 4);
    }

    pub fn set_blue(&mut self, local: (u8, u8, u8), value: u16) {
        let index = LightMap::get_index(local);
        self.map[index] = self.map[index] & 0xFFF0 | (value << 8);
    }

    pub fn set_sun(&mut self, local: (u8, u8, u8), value: u16) {
        let index = LightMap::get_index(local);
        self.map[index] = self.map[index] & 0xFFF0 | (value << 12);
    }

    pub fn set(&mut self, local: (u8, u8, u8), value: u16, channel: u8) {
        let index = LightMap::get_index(local);
        let color = self.map[index];
        self.map[index] = (color & (!(0xF << (channel*4)))) | (value << (channel << 2));
    }
}
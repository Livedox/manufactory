use std::{time::{Instant, Duration}, sync::Arc};
use crate::{world::global_coords::GlobalCoords, direction::Direction, voxels::chunks::Chunks, recipes::item::Item, bytes::{BytesCoder, AsFromBytes}};

#[derive(Debug)]
pub struct Manipulator {
    start_time: Option<Instant>,
    return_time: Option<Instant>,
    item_id: Option<u32>,
    direction: [i8; 3],
}


impl Manipulator {
    const SPEED: Duration = Duration::from_millis(300);

    pub fn new(direction: &Direction) -> Self {Self {
        start_time: None,
        return_time: None,
        item_id: None,
        direction: direction.simplify_to_one_greatest(true, false, true),
    }}

    pub fn update(&mut self, coords: GlobalCoords, chunks: &Chunks) {
        // let return_time = self.return_time.map_or(true, |rt| rt.elapsed() >= (Self::SPEED/2));
        // if self.item_id.is_none() && self.start_time.is_none() && return_time {
        //     let src_coords = GlobalCoords(coords.0 - self.direction[0] as i32, coords.1, coords.2 - self.direction[2] as i32);
        //     let Some(storage) = chunks.voxel_data(src_coords).and_then(|vd| vd.additionally.storage()) else {return};
        //     if let Some(item) = storage.lock().unwrap().take_first_existing(1) {
        //         self.item_id = Some(item.0.id());
        //         self.start_time = Some(Instant::now());
        //         self.return_time = None;
        //     };
        // }
        
        // let start_time = self.start_time.map_or(true, |rt| rt.elapsed() >= (Self::SPEED/2));
        // if self.item_id.is_some() && start_time {
        //     let dst_coords = GlobalCoords(coords.0 + self.direction[0] as i32, coords.1, coords.2 + self.direction[2] as i32);
        //     let Some(storage) = chunks.voxel_data(dst_coords).and_then(|vd| vd.additionally.storage()) else {return};
        //     let result = storage.lock().unwrap().add(&Item::new(self.item_id.unwrap(), 1), false).is_none();
        //     if result {
        //         self.item_id = None;
        //         self.start_time = None;
        //         self.return_time = Some(Instant::now());
        //     }
        // }
    }


    pub fn animation_progress(&self) -> f32 {
        if let Some(start_time) = self.start_time {
            (start_time.elapsed().as_secs_f32() / Self::SPEED.as_secs_f32()).min(0.5)
        } else if let Some(return_time) = self.return_time {
            (return_time.elapsed().as_secs_f32() / Self::SPEED.as_secs_f32() + 0.5).min(1.0)
        } else {
            0.0
        }
    }


    pub fn rotation_index(&self) -> u32 {
        if self.direction[0] < 0 {return 2};
        if self.direction[2] > 0 {return 3};
        if self.direction[2] < 0 {return 1};
        0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Header {
    item_id: u32,
    direction: [i8; 3],
}
impl AsFromBytes for Header {}

impl BytesCoder for Manipulator {
    fn decode_bytes(bytes: &[u8]) -> Self {
        let header = Header::from_bytes(&bytes[0..Header::size()]);
        Self {
            start_time: None,
            return_time: None,
            item_id: if u32::MAX == header.item_id {None} else {Some(header.item_id)},
            direction: header.direction,
        }
    }

    fn encode_bytes(&self) -> Box<[u8]> {
        Header {
            item_id: self.item_id.unwrap_or(u32::MAX),
            direction: self.direction
        }.as_bytes().into()
    }
}
use std::{collections::HashMap, rc::Rc, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, cell::UnsafeCell, ops::{Deref, DerefMut}, time::{SystemTime, UNIX_EPOCH}, io::Cursor, mem::MaybeUninit};

use itertools::iproduct;

use crate::{light::light_map::LightMap, direction::Direction, world::{local_coords::LocalCoords, chunk_coords::ChunkCoords, global_coords::GlobalCoords}, GAME_VERSION, bytes::{ConstByteInterpretation, DynByteInterpretation, any_as_u8_slice}};

use super::{voxel::{self, Voxel}, voxel_data::{VoxelData, VoxelAdditionalData}, chunks::Chunks, block::blocks::BLOCKS};
use crate::bytes::NumFromBytes;

pub const CHUNK_SIZE: usize = 32;
pub const HALF_CHUNK_SIZE: usize = CHUNK_SIZE/2;
pub const CHUNK_SQUARE: usize = CHUNK_SIZE.pow(2);
pub const CHUNK_VOLUME: usize = CHUNK_SIZE.pow(3);
pub const CHUNK_BIT_SHIFT: usize = CHUNK_SIZE.ilog2() as usize;
pub const CHUNK_BITS: usize = CHUNK_SIZE - 1_usize;

#[derive(Debug)]
pub struct Chunk {
    pub voxels: [voxel::Voxel; CHUNK_VOLUME],
    pub voxels_data: HashMap<usize, VoxelData>,
    modified: AtomicBool,
    pub lightmap: LightMap,
    pub xyz: ChunkCoords
}


impl Chunk {
    pub fn new(pos_x: i32, pos_y: i32, pos_z: i32) -> Chunk {
        let mut voxels = [Voxel::new(0); CHUNK_VOLUME];
        let voxels_data = HashMap::new();

        for (y, z, x) in iproduct!(0..CHUNK_SIZE, 0..CHUNK_SIZE, 0..CHUNK_SIZE) {
            let real_x = x as i32 + pos_x*CHUNK_SIZE as i32;
            let real_y = y as i32 + pos_y*CHUNK_SIZE as i32;
            let real_z = z as i32 + pos_z*CHUNK_SIZE as i32;
            if real_y as f64 <= ((real_x as f64 *0.3).sin() * 0.5 + 0.5) * 10. {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 7;
            }
            if real_y <= 2 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 5;
            }
            if z == 0 && y == 16 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 7;
            }
            if x == 0 && y == 0 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 7;
            }

            if real_z == 200 {
                voxels[(y*CHUNK_SIZE+z)*CHUNK_SIZE+x].id = 7;
            }
        }

        Chunk {
            voxels,
            xyz: ChunkCoords(pos_x, pos_y, pos_z),
            voxels_data,
            modified: AtomicBool::new(true),
            lightmap: LightMap::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|voxel| { voxel.id == 0 })
    }


    pub fn is_air(&self, coords: LocalCoords) -> bool {
        self.voxel(coords).id == 0
    }

    pub fn voxel(&self, local_coords: LocalCoords) -> &Voxel {
        &self.voxels[local_coords.index()]
    }

    fn mut_voxel(&mut self, local_coords: LocalCoords) -> &mut Voxel {
        &mut self.voxels[local_coords.index()]
    }

    pub fn modified(&self) -> bool {
        self.modified.load(Ordering::Acquire)
    }

    pub fn modify(&mut self, value: bool) {
        self.modified.store(value, Ordering::Release);
    }

    pub fn set_voxel_id(&mut self, local_coords: LocalCoords, id: u32, direction: Option<&Direction>) {
        self.voxels_data.remove(&local_coords.index());
        self.mut_voxel(local_coords).id = id;
        if BLOCKS()[id as usize].is_additional_data() {
            self.voxels_data.insert(local_coords.index(), VoxelData {
                id,
                global_coords: self.xyz.to_global(local_coords),
                additionally: Arc::new(VoxelAdditionalData::new(id, direction.unwrap_or(&Direction::new_x()))),
            });
        }
    }


    pub fn mut_voxel_data(&mut self, local_coords: LocalCoords) -> Option<&mut VoxelData> {
        self.voxels_data.get_mut(&local_coords.index())
    }


    pub fn voxel_data(&self, local_coords: LocalCoords) -> Option<&VoxelData> {
        self.voxels_data.get(&local_coords.index())
    }


    pub fn voxels_data(&self) -> &HashMap<usize, VoxelData> {
        &self.voxels_data
    }


    pub fn mut_voxels_data(&mut self) -> &mut HashMap<usize, VoxelData> {
        &mut self.voxels_data
    }


    pub fn add_voxel_data(&mut self, local_coords: LocalCoords, voxel_data: VoxelData) -> Option<VoxelData> {
        self.voxels_data.insert(local_coords.index(), voxel_data)
    }
}


impl ConstByteInterpretation for [Voxel; CHUNK_VOLUME] {
    fn from_bytes(data: &[u8]) -> Self {
        let mut v: [MaybeUninit<Voxel>; CHUNK_VOLUME] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        for (i, elem) in v[..].iter_mut().enumerate() {
            elem.write(Voxel::from_bytes(&data[4*i..(4*i+4)]));
        }

        unsafe { std::mem::transmute::<_, [Voxel; CHUNK_VOLUME]>(v) }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        unsafe { any_as_u8_slice(self) }.into()
        // let mut v = Vec::with_capacity(std::mem::size_of::<Self>());
        // for vox in self.iter() {
        //     v.extend(vox.to_bytes().as_ref());
        // }
        // v.into()
    }

    fn size(&self) -> u32 {
        32*CHUNK_VOLUME as u32
    }
}

impl DynByteInterpretation for Chunk {
    fn to_bytes(&self) -> Box<[u8]> {
        let now: u64 = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let mut v = Vec::new();
        v.extend(GAME_VERSION.to_le_bytes());
        v.extend(now.to_le_bytes());
        v.extend(self.xyz.to_bytes().as_ref());
        v.extend(self.voxels.to_bytes().as_ref());
        v.into()
    }
    fn from_bytes(data: &[u8]) -> Self {
        let game_version = u32::from_bytes(&data[0..4]);
        let time = u64::from_bytes(&data[4..12]);
        let xyz: ChunkCoords = ChunkCoords::from_bytes(&data[12..24]);
        let voxels = <[Voxel; CHUNK_VOLUME]>::from_bytes(&data[24..]);
        Self {
            voxels,
            voxels_data: HashMap::new(),
            modified: AtomicBool::new(true),
            lightmap: LightMap::new(),
            xyz: xyz,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::voxels::chunk::CHUNK_SIZE;

    #[test]
    fn correct_chunk_size() {
        assert!(CHUNK_SIZE > 1 && (CHUNK_SIZE & CHUNK_SIZE-1) == 0 && CHUNK_SIZE <= 32);
    }
}
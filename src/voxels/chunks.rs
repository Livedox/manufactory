use std::{collections::HashMap, sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, Ordering, AtomicI32}}, marker::PhantomPinned};

use itertools::iproduct;

use crate::{direction::Direction, world::{global_coords::GlobalCoords, local_coords::LocalCoords, chunk_coords::ChunkCoords}, vec_none, unsafe_mutex::UnsafeMutex, save_load::{WorldRegions, EncodedChunk}, bytes::BytesCoder, light::light_map::Light};

use super::{chunk::{Chunk, CHUNK_SIZE}, voxel::Voxel, voxel_data::{VoxelAdditionalData, VoxelData, multiblock::MultiBlock}};

pub const WORLD_HEIGHT: usize = 256 / CHUNK_SIZE; // In chunks

#[derive(Debug)]
pub struct Chunks {
    is_translate: AtomicBool,
    pub chunks: Arc<UnsafeMutex<Vec<Option<Arc<Chunk>>>>>,
    pub chunks_awaiting_deletion: Arc<Mutex<Vec<Arc<Chunk>>>>,

    pub volume: i32,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    
    pub ox: AtomicI32,
    pub oz: AtomicI32,
    
    width_with_offset: AtomicI32, //Needed to optimize the function (is_in_area)
    depth_with_offset: AtomicI32, //Needed to optimize the function (is_in_area)
}

impl Chunks {
    pub fn new(width: i32, height: i32, depth: i32, ox: i32, oy: i32, oz: i32) -> Chunks {
        let volume = width*height*depth;
        let mut chunks: Vec<Option<Arc<Chunk>>> = vec![];
        for _ in 0..volume { chunks.push(None); }

        Chunks {
            chunks_awaiting_deletion: Arc::new(Mutex::new(Vec::new())),
            chunks: Arc::new(UnsafeMutex::new(chunks)),
            volume,
            width,
            height,
            depth,
            
            width_with_offset: AtomicI32::new(width+ox),
            depth_with_offset: AtomicI32::new(depth+oz),
            ox: AtomicI32::new(ox),
            oz: AtomicI32::new(oz),
            is_translate: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn is_translate(&self) -> bool {
        self.is_translate.load(Ordering::Relaxed)
    }
    #[inline]
    pub fn set_translate(&self, value: bool) {
        self.is_translate.store(value, Ordering::Relaxed);
    }


    #[inline] pub fn ox(&self) -> i32 {self.ox.load(Ordering::Relaxed)}
    #[inline] pub fn oz(&self) -> i32 {self.oz.load(Ordering::Relaxed)}
    #[inline] pub fn set_ox(&self, value: i32) {self.ox.store(value, Ordering::Relaxed)}
    #[inline] pub fn set_oz(&self, value: i32) {self.oz.store(value, Ordering::Relaxed)}

    #[inline] pub fn width_with_offset(&self) -> i32 {
        self.width_with_offset.load(Ordering::Relaxed)}
    #[inline] pub fn depth_with_offset(&self) -> i32 {
        self.depth_with_offset.load(Ordering::Relaxed)}
    #[inline] pub fn set_width_with_offset(&self, value: i32) {
        self.width_with_offset.store(value, Ordering::Relaxed)}
    #[inline] pub fn set_depth_with_offset(&self, value: i32) {
        self.depth_with_offset.store(value, Ordering::Relaxed)}

    /// ONLY SAFE ACCESS
    pub fn translate(&self, ox: i32, oz: i32) -> Vec<(usize, usize)> {
        let mut indices = Vec::<(usize, usize)>::new();
        let mut chunks = self.chunks.lock().unwrap();
        let mut new_chunks: Vec<Option<Arc<Chunk>>> = vec_none!(chunks.len());

        let dx = ox - self.ox();
        let dz = oz - self.oz();
        for (cz, cx, cy) in iproduct!(0..self.depth, 0..self.width, 0..self.height) {
            let nx = cx - dx;
            let nz = cz - dz;
            if nx < 0 || nz < 0 || nx >= self.width || nz >= self.depth {continue};

            let new_index = ChunkCoords(nx, cy, nz).index_without_offset(self.width, self.depth);
            let old_index = ChunkCoords(cx, cy, cz).index_without_offset(self.width, self.depth);
            
            indices.push((old_index, new_index));
            new_chunks[new_index] = chunks[old_index].take();
        }

        for chunk in chunks.iter_mut() {
            let Some(chunk) = chunk.take() else {continue};
            if chunk.unsaved() {self.chunks_awaiting_deletion.lock().unwrap().push(chunk)}
        }

        chunks.clear();
        *chunks = new_chunks;
        self.set_ox(ox);
        self.set_oz(oz);
        self.set_width_with_offset(self.width + ox);
        self.set_depth_with_offset(self.depth + oz);
        indices
    }


    pub fn load_all(&self, world_regions: Arc<UnsafeMutex<WorldRegions>>) {
        for (cy, cz, cx) in iproduct!(0..self.height, 0..self.depth, 0..self.width) {
            let index = ChunkCoords(cx, cy, cz).index_without_offset(self.width, self.depth);
            let mut chunks = unsafe {self.chunks.lock_unsafe()}.unwrap();
            let Some(chunk) = chunks.get_mut(index) else {continue};
            let mut world_regions = unsafe {world_regions.lock_unsafe()}.unwrap();
            *chunk = match world_regions.chunk(ChunkCoords(cx+self.ox(), cy, cz+self.oz())) {
                EncodedChunk::None => Some(Arc::new(Chunk::new(cx+self.ox(), cy, cz+self.oz()))),
                EncodedChunk::Some(b) => Some(Arc::new(Chunk::decode_bytes(b))),
            }
        }
    }

    pub fn voxel(&self, chunk_coords: ChunkCoords, local_coords: LocalCoords) -> Option<Voxel> {
        self.chunk(chunk_coords).map(|c| c.voxel(local_coords))
    }

    pub fn voxel_global(&self, coords: GlobalCoords) -> Option<Voxel> {
        self.voxel(coords.into(), coords.into())
    }

    pub fn is_air_global(&self, coords: GlobalCoords) -> bool {
        let Some(voxel) = self.voxel(coords.into(), coords.into()) else {return false};
        voxel.id == 0
    }


    pub fn set(&self, global: GlobalCoords, id: u32, direction: Option<&Direction>) -> Option<u32> {
        let coords: ChunkCoords = global.into();
        let Some(chunk) = self.chunk(coords) else {return None};

        let local: LocalCoords = global.into();
    
        let x_offset = (local.0 == (CHUNK_SIZE-1) as u8) as i32 - (local.0 == 0) as i32;
        let y_offset = (local.1 == (CHUNK_SIZE-1) as u8) as i32 - (local.1 == 0) as i32;
        let z_offset = (local.2 == (CHUNK_SIZE-1) as u8) as i32 - (local.2 == 0) as i32;
        chunk.set_voxel_id(local, id, direction);
        chunk.modify(true);
        chunk.save(true);
        
        if x_offset != 0 {
            if let Some(chunk) = self.chunk((coords.0+x_offset, coords.1, coords.2)) {chunk.modify(true)};
        }
        if y_offset != 0 {
            if let Some(chunk) = self.chunk((coords.0, coords.1+y_offset, coords.2)) {chunk.modify(true)};
        }
        if z_offset != 0 {
            if let Some(chunk) = self.chunk((coords.0, coords.1, coords.2+z_offset)) {chunk.modify(true)};
        }
        
        Some(id)
    }

    #[inline]
    pub fn is_in_area(&self, chunk_coords: ChunkCoords) -> bool {
        chunk_coords.0 >= self.ox() && chunk_coords.0 < self.width_with_offset() &&
        chunk_coords.1 >= 0 && chunk_coords.1 < self.height &&
        chunk_coords.2 >= self.oz() && chunk_coords.2 < self.depth_with_offset()
    }

    pub fn local_chunk(&self, coords: ChunkCoords) -> Option<Arc<Chunk>> {
        let index = coords.index_without_offset(self.width, self.depth);
        unsafe {self.chunks.lock_unsafe()}.unwrap().get(index).and_then(|c| c.as_ref().map(|c| c.clone()))
    }

    pub fn chunk<T: Into<ChunkCoords>>(&self, coords: T) -> Option<Arc<Chunk>> {
        let coords: ChunkCoords = coords.into();
        if !self.is_in_area(coords) { return None; }
        let index = coords.nindex(self.width, self.depth, self.ox(), self.oz());
        // It's safe because we checked the coordinates
        unsafe {self.chunks.lock_unsafe().unwrap().get_unchecked(index).as_ref().map(|c| c.clone())}
    }

    pub fn voxels_data<T: Into<ChunkCoords>>(&self, coords: T) -> Option<Arc<RwLock<HashMap<usize, Arc<VoxelData>>>>> {
        self.chunk(coords).map(|c| c.voxels_data())
    }

    pub fn voxel_data(&self, gc: GlobalCoords) -> Option<Arc<VoxelData>> {
        let voxel_data = self.voxels_data(gc).and_then(|vd| {
            vd.read().unwrap().get(&LocalCoords::from(gc).index()).map(|d| d.clone())
        });
        let Some(VoxelAdditionalData::MultiBlockPart(gc)) = voxel_data.as_ref().map(|vd| vd.additionally.as_ref()) else {
            return voxel_data;
        };
        return self.voxels_data(*gc).and_then(|vd| {
            vd.read().unwrap().get(&LocalCoords::from(*gc).index()).map(|d| d.clone())
        });
    }

    pub fn set_additional_voxel_data(&self, id: u32, gc: GlobalCoords, ad: Arc<VoxelAdditionalData>) {
        let local: LocalCoords = gc.into();
        let vd = self.voxels_data(gc);

        if let Some(vd) = vd {
            vd.write().unwrap().insert(local.index(), Arc::new(VoxelData { id, global_coords: gc, additionally: ad }));
        }
    }


    pub fn add_multiblock_structure(&self, xyz: &GlobalCoords, width: i32, height: i32, depth: i32, id: u32, dir: &Direction) -> Option<Vec<GlobalCoords>> {
        let mut coords: Vec<GlobalCoords> = vec![];
        // FIX THIS SHIT
        let width_range = if width > 0 {
            (xyz.0)..(xyz.0+width)
        } else {
            (xyz.0+width+1)..(xyz.0+1)
        };
        let height_range = if height > 0 {
            (xyz.1)..(xyz.1+height)
        } else {
            (xyz.1+height+1)..(xyz.1+1)
        };
        let depth_range = if depth > 0 {
            (xyz.2)..(xyz.2+depth)
        } else {
            (xyz.2+depth+1)..(xyz.2+1)
        };
        coords.push(*xyz);
        for (nx, nz, ny) in iproduct!(width_range, depth_range, height_range) {
            if nx == xyz.0 && ny == xyz.1 && nz == xyz.2 {continue};
            if !self.is_air_global((nx, ny, nz).into()) {return None};
            coords.push((nx, ny, nz).into());
        }

        self.set(coords[0], id, None);
        let voxels_data = self.voxels_data(coords[0]).unwrap();
        voxels_data.write().unwrap().insert(LocalCoords::from(coords[0]).index(), Arc::new(VoxelData {
            id,
            global_coords: coords[0],
            additionally: Arc::new(VoxelAdditionalData::new_multiblock(id, dir, coords.clone())),
        }));
        coords.iter().skip(1).for_each(|coord| {
            self.set(*coord, 1, None);
            let voxels_data = self.voxels_data(*coord).unwrap();
            voxels_data.write().unwrap().insert(LocalCoords::from(*coord).index(), Arc::new(VoxelData {
                id: 1,
                global_coords: *coord,
                additionally: Arc::new(VoxelAdditionalData::MultiBlockPart(coords[0])),
            }));
        });
        Some(coords)
    }


    pub fn remove_multiblock_structure(&self, global: GlobalCoords) -> Option<Vec<GlobalCoords>> {
        let Some(voxel_data) = self.voxel_data(global) else {return None};
        let mut coords: Vec<GlobalCoords> = vec![];
        match &voxel_data.additionally.as_ref() {
            VoxelAdditionalData::Drill(drill) => {
                drill.lock().unwrap().structure_coordinates().iter().for_each(|coord| {
                    coords.push(*coord);
                });
            },
            VoxelAdditionalData::AssemblingMachine(asembler) => {
                asembler.lock().unwrap().structure_coordinates().iter().for_each(|coord| {
                    coords.push(*coord);
                });
            },
            _ => (),
        };
        coords.iter().for_each(|coord| {
            self.set(*coord, 0, None);
        });
        Some(coords)
    }

    pub fn get_sun(&self, coords: GlobalCoords) -> u16 {
        let local: LocalCoords = coords.into();
        self.chunk(coords)
            .map_or(0, |c| c.lightmap.get_sun(local.into()))
    }

    pub fn light(&self, coords: GlobalCoords, channel: u8) -> u16 {
        self.chunk(coords).map_or(0, |c| c.lightmap.get(LocalCoords::from(coords).into(), channel))
    }

    pub fn get_light(&self, coords: GlobalCoords) -> Light {
        let local: LocalCoords = coords.into();
        self.chunk(coords)
            .map_or(Light::default(), |c| c.lightmap.get_light(local.into()))
    }
}

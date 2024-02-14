use std::{collections::HashMap, sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, Ordering, AtomicI32}}, marker::PhantomPinned, cell::UnsafeCell};

use itertools::iproduct;

use crate::{bytes::BytesCoder, content::Content, direction::Direction, light::light_map::Light, save_load::{WorldRegions, EncodedChunk}, unsafe_mutex::UnsafeMutex, vec_none, world::{global_coords::GlobalCoords, local_coords::LocalCoords, chunk_coords::ChunkCoords}};

use super::{chunk::{Chunk, LiveVoxels, CHUNK_SIZE}, live_voxels::{mutliblock_part::MultiBlockPart, LiveVoxel, LiveVoxelContainer}, voxel::Voxel, voxel_data::{multiblock::MultiBlock, VoxelAdditionalData, VoxelData}};

pub const WORLD_BLOCK_HEIGHT: usize = 256;
pub const WORLD_HEIGHT: usize = WORLD_BLOCK_HEIGHT / CHUNK_SIZE; // In chunks

#[derive(Debug)]
pub struct Chunks {
    pub content: Arc<Content>,
    is_translate: AtomicBool,
    // I tried to do this using safe code, but it kills performance by about 2 times
    pub chunks: UnsafeCell<Vec<Option<Arc<Chunk>>>>,
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
    pub fn new(content: Arc<Content>, width: i32, height: i32, depth: i32, ox: i32, oy: i32, oz: i32) -> Chunks {
        let volume = width*height*depth;
        let mut chunks: Vec<Option<Arc<Chunk>>> = vec![];
        for _ in 0..volume { chunks.push(None); }

        Chunks {
            content,
            chunks: UnsafeCell::new(chunks),
            chunks_awaiting_deletion: Arc::new(Mutex::new(Vec::new())),
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
    
    pub fn translate(&self, ox: i32, oz: i32) -> Vec<(usize, usize)> {
        let mut indices = Vec::<(usize, usize)>::new();
        let chunks = unsafe {&mut *self.chunks.get()};
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

    pub fn voxel(&self, chunk_coords: ChunkCoords, local_coords: LocalCoords) -> Option<Voxel> {
        let chunk = self.chunk_ptr(chunk_coords)?;
        Some(unsafe {&*chunk}.voxel(local_coords))
    }

    pub fn voxel_global(&self, coords: GlobalCoords) -> Option<Voxel> {
        self.voxel(coords.into(), coords.into())
    }

    pub fn is_air_global(&self, coords: GlobalCoords) -> bool {
        let Some(voxel) = self.voxel(coords.into(), coords.into()) else {return false};
        voxel.id == 0
    }


    pub fn set_block(&self, global: GlobalCoords, id: u32, direction: Option<&Direction>) {
        self.set(global, id);
        let Some(live_voxels) = self.voxels_data(global) else {return};
        let block = &self.content.blocks[id as usize].base;
        let Some(name) = &block.live_voxel else {return};
        let local: LocalCoords = global.into();
        // live_voxels.insert(local.index(), self.content.live_voxel.new);
    }


    pub fn set(&self, global: GlobalCoords, id: u32) {
        let coords: ChunkCoords = global.into();
        let Some(chunk) = self.chunk(coords) else {return};

        let local: LocalCoords = global.into();
    
        let x_offset = (local.0 == (CHUNK_SIZE-1) as u8) as i32 - (local.0 == 0) as i32;
        let y_offset = (local.1 == (CHUNK_SIZE-1) as u8) as i32 - (local.1 == 0) as i32;
        let z_offset = (local.2 == (CHUNK_SIZE-1) as u8) as i32 - (local.2 == 0) as i32;
        chunk.set_voxel_id(local, id, &self.content);
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
    }

    #[inline]
    pub fn is_in_area(&self, chunk_coords: ChunkCoords) -> bool {
        chunk_coords.0 >= self.ox() && chunk_coords.0 < self.width_with_offset() &&
        chunk_coords.1 >= 0 && chunk_coords.1 < self.height &&
        chunk_coords.2 >= self.oz() && chunk_coords.2 < self.depth_with_offset()
    }

    pub fn local_chunk(&self, coords: ChunkCoords) -> Option<Arc<Chunk>> {
        let index = coords.index_without_offset(self.width, self.depth);
        unsafe {&mut *self.chunks.get()}.get(index).and_then(|c| c.as_ref().map(|c| c.clone()))
    }

    pub fn chunk<T: Into<ChunkCoords>>(&self, coords: T) -> Option<Arc<Chunk>> {
        let coords: ChunkCoords = coords.into();
        if !self.is_in_area(coords) { return None; }
        let index = coords.nindex(self.width, self.depth, self.ox(), self.oz());
        // It's safe because we checked the coordinates
        let lock = unsafe {&mut *self.chunks.get()};
        let r = unsafe {lock.get_unchecked(index)}.as_ref();
        r.map(|c| c.clone())
    }

    pub fn chunk_ptr<T: Into<ChunkCoords>>(&self, coords: T) -> Option<*const Chunk> {
        let coords: ChunkCoords = coords.into();
        if !self.is_in_area(coords) { return None; }
        let index = coords.nindex(self.width, self.depth, self.ox(), self.oz());
        // It's safe because we checked the coordinates
        let lock = unsafe {&mut *self.chunks.get()};
        let r = unsafe {lock.get_unchecked(index)}.as_ref();
        r.map(|c| c.as_ref() as *const Chunk)
    }

    #[inline]
    pub unsafe fn get_unchecked_chunk(&self, index: usize) -> Option<*const Chunk> {
        unsafe {(&*self.chunks.get()).get_unchecked(index)}
            .as_ref().map(|c| c.as_ref() as *const Chunk)
    }

    pub fn voxels_data<T: Into<ChunkCoords>>(&self, coords: T) -> Option<LiveVoxels> {
        self.chunk(coords).map(|c| c.voxels_data())
    }

    pub fn voxel_data(&self, gc: GlobalCoords) -> Option<Arc<LiveVoxelContainer>> {
        let voxel_data = self.voxels_data(gc).and_then(|vd| {
            vd.get(&LocalCoords::from(gc).index())
        });
        if let Some(Some(gc)) = voxel_data.as_ref().map(|vd| {
            vd.live_voxel.parent_coord()
        }) {
            return self.voxels_data(gc).and_then(|vd| {
                vd.get(&LocalCoords::from(gc).index())
            });
        };
        return voxel_data;
    }

    pub fn add_multiblock_structure(&self, xyz: &GlobalCoords, width: i32, height: i32, depth: i32, id: u32, dir: &Direction) -> Option<Vec<GlobalCoords>> {
        let mut coords: Vec<GlobalCoords> = vec![];
        //FIX THIS SHIT
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
        self.set(coords[0], id);

        let live_voxel_name = self.content.blocks[id as usize].live_voxel().unwrap_or("multiblock");
        let voxels_data = self.voxels_data(coords[0]).unwrap();
        let live_voxel: Box<(dyn LiveVoxel)> = self.content.live_voxel.new_multiblock.get(live_voxel_name)
            .map_or(Box::new(()), |f| { f(dir, coords.clone())});

        voxels_data.insert(LocalCoords::from(coords[0]).index(), 
            LiveVoxelContainer::new_arc(id, coords[0].into(), live_voxel));
        
        coords.iter().skip(1).for_each(|coord| {
            self.set(*coord, 1);
            let voxels_data = self.voxels_data(*coord).unwrap();
            voxels_data.insert(LocalCoords::from(*coord).index(),
                LiveVoxelContainer::new_arc_multiblock_part((*coord).into(), coords[0]));
        });
        Some(coords)
    }


    pub fn remove_multiblock_structure(&self, global: GlobalCoords) -> Option<Vec<GlobalCoords>> {
        let Some(voxel_data) = self.voxel_data(global) else {return None};
        let coords = voxel_data.structure_coordinates().unwrap();
        coords.iter().for_each(|coord| {
            self.set(*coord, 0);
        });
        Some(coords)
    }

    pub fn get_sun(&self, coords: GlobalCoords) -> u8 {
        self.chunk(coords)
            .map_or(0, |c| c.lightmap.get(coords.into()).get_sun())
    }

    #[inline(never)]
    pub fn light(&self, coords: GlobalCoords, channel: usize) -> u8 {
        let Some(chunk) = self.chunk_ptr(coords) else {return 0};
        unsafe {&*chunk}.lightmap.get(LocalCoords::from(coords).into()).get_channel(channel)
    }

    pub fn get_light(&self, coords: GlobalCoords) -> Light {
        let Some(chunk) = self.chunk_ptr(coords) else {return Light::default()};
        unsafe {&*chunk}.lightmap.get(coords.into()).clone()
    }
}

unsafe impl Sync for Chunks {}
unsafe impl Send for Chunks {}
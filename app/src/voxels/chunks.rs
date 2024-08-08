use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering, AtomicI32}}, cell::UnsafeCell};

use itertools::{iproduct, Itertools};

use crate::{content::Content, direction::Direction, light::light_map::Light, vec_none, coords::{global_coord::GlobalCoord, local_coord::LocalCoord, chunk_coord::ChunkCoord}};

use super::{chunk::{Chunk, LiveVoxels, CHUNK_SIZE}, live_voxels::{LiveVoxelBehavior, LiveVoxelContainer}, voxel::Voxel};

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
    pub fn new(content: Arc<Content>, width: i32, height: i32, depth: i32, ox: i32, _oy: i32, oz: i32) -> Chunks {
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

            let new_index = ChunkCoord::new(nx, cy, nz).index_without_offset(self.width, self.depth);
            let old_index = ChunkCoord::new(cx, cy, cz).index_without_offset(self.width, self.depth);
            
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

    pub fn voxel(&self, chunk_coords: ChunkCoord, local_coords: LocalCoord) -> Option<Voxel> {
        let chunk = self.chunk_ptr(chunk_coords)?;
        Some(unsafe {&*chunk}.voxel(local_coords))
    }

    pub fn voxel_global(&self, coords: GlobalCoord) -> Option<Voxel> {
        self.voxel(coords.into(), coords.into())
    }

    pub fn is_air_global(&self, coords: GlobalCoord) -> bool {
        let Some(voxel) = self.voxel(coords.into(), coords.into()) else {return false};
        voxel.id == 0
    }


    pub fn set_block(&self, global: GlobalCoord, id: u32, direction: Option<&Direction>) {
        self.set_voxel(global, id);
        let Some(live_voxels) = self.live_voxels(global) else {return};
        let block = &self.content.blocks[id as usize].base;
        let Some(name) = &block.live_voxel else {return};
        let local: LocalCoord = global.into();
        println!("{:?}", name);
        let live_voxel = self.content.live_voxel.new.get(name).unwrap()(direction.unwrap_or(&Direction::new_x()));
        live_voxels.insert(local.index(), LiveVoxelContainer::new_arc(id, global, live_voxel));
    }


    pub fn set_voxel(&self, global: GlobalCoord, id: u32) {
        let coords: ChunkCoord = global.into();
        let Some(chunk) = self.chunk(coords) else {return};

        let local: LocalCoord = global.into();
    
        let x_offset = (local.x == (CHUNK_SIZE-1) as u8) as i32 - (local.x == 0) as i32;
        let y_offset = (local.y == (CHUNK_SIZE-1) as u8) as i32 - (local.y == 0) as i32;
        let z_offset = (local.z == (CHUNK_SIZE-1) as u8) as i32 - (local.z == 0) as i32;
        chunk.set_voxel_id(local, id);
        chunk.modify(true);
        chunk.save(true);
        
        if x_offset != 0 {
            if let Some(chunk) = self.chunk((coords.x+x_offset, coords.y, coords.z)) {chunk.modify(true)};
        }
        if y_offset != 0 {
            if let Some(chunk) = self.chunk((coords.x, coords.y+y_offset, coords.z)) {chunk.modify(true)};
        }
        if z_offset != 0 {
            if let Some(chunk) = self.chunk((coords.x, coords.y, coords.z+z_offset)) {chunk.modify(true)};
        }
    }

    #[inline]
    pub fn is_in_area(&self, chunk_coords: ChunkCoord) -> bool {
        chunk_coords.x >= self.ox() && chunk_coords.x < self.width_with_offset() &&
        chunk_coords.y >= 0 && chunk_coords.y < self.height &&
        chunk_coords.z >= self.oz() && chunk_coords.z < self.depth_with_offset()
    }

    pub fn local_chunk(&self, coords: ChunkCoord) -> Option<Arc<Chunk>> {
        let index = coords.index_without_offset(self.width, self.depth);
        unsafe {&mut *self.chunks.get()}.get(index).and_then(|c| c.as_ref().cloned())
    }

    pub fn chunk<T: Into<ChunkCoord>>(&self, coords: T) -> Option<Arc<Chunk>> {
        let coords: ChunkCoord = coords.into();
        if !self.is_in_area(coords) { return None; }
        let index = coords.nindex(self.width, self.depth, self.ox(), self.oz());
        // It's safe because we checked the coordinates
        let lock = unsafe {&mut *self.chunks.get()};
        let r = unsafe {lock.get_unchecked(index)}.as_ref();
        r.cloned()
    }

    pub fn chunk_ptr<T: Into<ChunkCoord>>(&self, coords: T) -> Option<*const Chunk> {
        let coords: ChunkCoord = coords.into();
        if !self.is_in_area(coords) { return None; }
        let index = coords.nindex(self.width, self.depth, self.ox(), self.oz());
        // It's safe because we checked the coordinates
        let lock = unsafe {&mut *self.chunks.get()};
        let r = unsafe {lock.get_unchecked(index)}.as_ref();
        r.map(|c| c.as_ref() as *const Chunk)
    }

    #[inline]
    pub unsafe fn get_unchecked_chunk(&self, index: usize) -> Option<*const Chunk> {
        unsafe {(*self.chunks.get()).get_unchecked(index)}
            .as_ref().map(|c| c.as_ref() as *const Chunk)
    }

    pub fn live_voxels<T: Into<ChunkCoord>>(&self, coords: T) -> Option<LiveVoxels> {
        self.chunk(coords).map(|c| c.live_voxels())
    }

    pub fn master_live_voxel(&self, gc: GlobalCoord) -> Option<Arc<LiveVoxelContainer>> {
        let live_voxels = self.live_voxels(gc)?;
        let live_voxel = live_voxels.get(&LocalCoord::from(gc).index())?;

        if let Some(master_coord) = live_voxel.master_coord() {
            let index = LocalCoord::from(master_coord).index();
            return self.live_voxels(master_coord)?.get(&index);
        }

        Some(live_voxel)
    }

    pub fn add_multiblock_structure(&self, xyz: &GlobalCoord, width: i32, height: i32, depth: i32, id: u32, dir: &Direction) -> Option<Vec<GlobalCoord>> {
        let mut coords: Vec<GlobalCoord> = vec![];
        //FIX THIS SHIT
        let width_range = if width > 0 {
            (xyz.x)..(xyz.x+width)
        } else {
            (xyz.x+width+1)..(xyz.x+1)
        };
        let height_range = if height > 0 {
            (xyz.y)..(xyz.y+height)
        } else {
            (xyz.y+height+1)..(xyz.y+1)
        };
        let depth_range = if depth > 0 {
            (xyz.z)..(xyz.z+depth)
        } else {
            (xyz.z+depth+1)..(xyz.z+1)
        };
        coords.push(*xyz);
        for (nx, nz, ny) in iproduct!(width_range, depth_range, height_range) {
            if nx == xyz.x && ny == xyz.y && nz == xyz.z {continue};
            if !self.is_air_global((nx, ny, nz).into()) {return None};
            coords.push((nx, ny, nz).into());
        }
        self.set_voxel(coords[0], id);

        let live_voxel_name = self.content.blocks[id as usize].live_voxel().unwrap_or("");
        let voxels_data = self.live_voxels(coords[0]).unwrap();
        let live_voxel: Box<(dyn LiveVoxelBehavior)> = self.content.live_voxel.new.get(live_voxel_name)
            .map_or(Box::new(()), |f| { f(dir)});

        voxels_data.insert(LocalCoord::from(coords[0]).index(), 
            LiveVoxelContainer::new_arc_master(id, coords[0], coords.clone(), live_voxel));
        
        coords.iter().skip(1).for_each(|coord| {
            self.set_voxel(*coord, 1);
            let voxels_data = self.live_voxels(*coord).unwrap();
            voxels_data.insert(LocalCoord::from(*coord).index(),
                LiveVoxelContainer::new_arc_slave(*coord, coords[0]));
        });
        Some(coords)
    }


    pub fn remove_multiblock_structure(&self, global: GlobalCoord) -> Option<Vec<GlobalCoord>> {
        let live_voxel = self.master_live_voxel(global)?;
        let coords = live_voxel.multiblock_coords().unwrap();
        coords.iter().for_each(|coord| {
            self.set_voxel(*coord, 0);
        });
        Some(coords.iter().copied().collect_vec())
    }

    pub fn get_sun(&self, coords: GlobalCoord) -> u8 {
        self.chunk(coords)
            .map_or(0, |c| c.lightmap.get(coords.into()).get_sun())
    }

    pub fn light(&self, coords: GlobalCoord, channel: usize) -> u8 {
        let Some(chunk) = self.chunk_ptr(coords) else {return 0};
        unsafe {&*chunk}.lightmap.get(LocalCoord::from(coords)).get_channel(channel)
    }

    pub fn get_light(&self, coords: GlobalCoord) -> Light {
        let Some(chunk) = self.chunk_ptr(coords) else {return Light::default()};
        unsafe {&*chunk}.lightmap.get(coords.into()).clone()
    }
}

unsafe impl Sync for Chunks {}
unsafe impl Send for Chunks {}
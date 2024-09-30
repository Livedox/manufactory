use std::{cell::UnsafeCell, collections::HashMap, hash::{BuildHasher, Hash}, ops::{Add, AddAssign, Sub}, sync::{atomic::{AtomicBool, AtomicI32, Ordering}, Arc, Mutex}};

use itertools::{iproduct, Itertools};
use serde::{Deserialize, Serialize};

use crate::{content::Content, coords::{chunk_coord::{ChunkCoord, MyHasher}, coord::Coord, global_coord::GlobalCoord, local_coord::LocalCoord}, direction::Direction, light::light::Light, vec_none};

use super::{live_voxels::{LiveVoxelBehavior, LiveVoxelContainer}, chunk::{Chunk, LiveVoxels, CHUNK_BITS, CHUNK_BIT_SHIFT, CHUNK_SIZE}, voxel::Voxel};

pub const WORLD_BLOCK_HEIGHT: usize = 256;
pub const WORLD_HEIGHT: usize = WORLD_BLOCK_HEIGHT / CHUNK_SIZE; // In chunks
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct Chunks {
    pub content: Arc<Content>,
    is_translate: AtomicBool,
    // I tried to do this using safe code, but it kills performance by about 2 times
    pub chunks: UnsafeCell<FxHashMap<ChunkCoord, Arc<Chunk>>>,
    pub chunks_awaiting_deletion: Arc<Mutex<Vec<Arc<Chunk>>>>,
    
    pub ox: AtomicI32,
    pub oz: AtomicI32,

    pub render_radius: i32,
}

impl Chunks {
    pub fn new(content: Arc<Content>, render_radius: i32, ox: i32, oz: i32) -> Chunks {
        Chunks {
            content,
            chunks: UnsafeCell::new(FxHashMap::default()),
            chunks_awaiting_deletion: Arc::new(Mutex::new(Vec::new())),
            render_radius,
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

    pub fn voxel(&self, cc: ChunkCoord, lc: LocalCoord) -> Option<Voxel> {
        self.chunk(cc)?.voxels().get(lc).map(|v| v.to_voxel())
    }

    pub fn voxel_global(&self, gc: GlobalCoord) -> Option<Voxel> {
        if gc.y < 0 || gc.y >= WORLD_BLOCK_HEIGHT as i32 {return None};
        self.voxel(gc.into(), gc.into())
    }

    pub fn set_block(&self, global: GlobalCoord, id: u32, direction: Option<&Direction>) {
        self.set_voxel(global, id);
        let Some(live_voxels) = self.live_voxels(global.into()) else {return};
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
    
        let x_offset = (local.x == (CHUNK_SIZE-1)) as i32 - (local.x == 0) as i32;
        let z_offset = (local.z == (CHUNK_SIZE-1)) as i32 - (local.z == 0) as i32;
        chunk.set_voxel_id(local, id);
        chunk.modify(true);
        chunk.save(true);
        
        if x_offset != 0 {
            if let Some(chunk) = self.chunk((coords.x+x_offset, coords.z).into()) {chunk.modify(true)};
        }
        if z_offset != 0 {
            if let Some(chunk) = self.chunk((coords.x, coords.z+z_offset).into()) {chunk.modify(true)};
        }
    }

    // UPDATE
    #[inline(never)]
    pub fn chunk(&self, cc: ChunkCoord) -> Option<&Arc<Chunk>> {
        let lock = unsafe {&mut *self.chunks.get()};
        lock.get(&cc)
    }

    pub fn live_voxels(&self, cc: ChunkCoord) -> Option<LiveVoxels> {
        self.chunk(cc).map(|c| c.live_voxels())
    }

    pub fn master_live_voxel(&self, gc: GlobalCoord) -> Option<Arc<LiveVoxelContainer>> {
        let live_voxels = self.live_voxels(gc.into())?;
        let live_voxel = live_voxels.get(&LocalCoord::from(gc).index())?;

        if let Some(master_coord) = live_voxel.master_coord() {
            let index = LocalCoord::from(master_coord).index();
            return self.live_voxels(master_coord.into())?.get(&index);
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
            if self.voxel_global((nx, ny, nz).into()).map_or(false, |v| v.id != 0) {return None};
            coords.push((nx, ny, nz).into());
        }
        self.set_voxel(coords[0], id);

        let live_voxel_name = self.content.blocks[id as usize].live_voxel().unwrap_or("");
        let voxels_data = self.live_voxels(coords[0].into()).unwrap();
        let live_voxel: Box<(dyn LiveVoxelBehavior)> = self.content.live_voxel.new.get(live_voxel_name)
            .map_or(Box::new(()), |f| { f(dir)});

        voxels_data.insert(LocalCoord::from(coords[0]).index(), 
            LiveVoxelContainer::new_arc_master(id, coords[0], coords.clone(), live_voxel));
        
        coords.iter().skip(1).for_each(|coord| {
            self.set_voxel(*coord, 1);
            let voxels_data = self.live_voxels((*coord).into()).unwrap();
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

    pub fn get_sun(&self, gc: GlobalCoord) -> u8 {
        self.chunk(gc.into())
            .map_or(0, |c| c.lightmap[gc.into()].get_sun())
    }

    pub fn light(&self, gc: GlobalCoord, channel: usize) -> u8 {
        let Some(chunk) = self.chunk(gc.into()) else {return 0};
        chunk.lightmap[gc.into()].get_channel(channel)
    }

    pub fn get_light(&self, gc: GlobalCoord) -> Light {
        let Some(chunk) = self.chunk(gc.into()) else {return Light::default()};
        if gc.y < 0 || gc.y >= WORLD_BLOCK_HEIGHT as i32 {return Light::default()};
        chunk.lightmap[gc.into()].clone()
    }
}

unsafe impl Sync for Chunks {}
unsafe impl Send for Chunks {}

const SIDE_COORDS_OFFSET: [(i32, i32, i32); 4] = [
    (1,0,0), (-1,0,0),
    (0,0,1), (0,0,-1),
];

impl Chunks {
    pub fn find_unloaded(&self) -> Option<(i32, i32)> {
        let callback = |cx: i32, cz: i32| {
            self.chunk((cx, cz).into()).is_none().then_some((cx, cz))
        };
 
        Self::clockwise_square_spiral(self.render_radius as usize * 2, callback)
    }

    pub fn find_unrendered(&self) -> Option<Arc<Chunk>> {
        for chunk in unsafe {&*self.chunks.get()}.values() {
            if chunk.modified() {return Some(Arc::clone(chunk))}
        }
        None
        // let callback = |cx: i32, cz: i32| {
        //     let cc = (cx+1, cz+1).into();
        //     if self.chunk(cc)
        //         .map_or(true, |c| !c.modified()) {return None};

        //     let mut around_count = 0;
        //     for (ox, _, oz) in SIDE_COORDS_OFFSET.into_iter() {
        //         let cc = ChunkCoord::new(cx + ox + 1, cz + oz + 1);
        //         if self.chunk(cc).is_some() {around_count += 1}
        //     }
        //     if around_count == 4 {return Some(cc)}
        //     None
        // };

        // Self::clockwise_square_spiral(self.width as usize - 2, callback)
        //     .and_then(|cc| (self.chunk(cc).cloned()))
    }

    pub fn clockwise_square_spiral<T>(n: usize, callback: impl Fn(i32, i32) -> Option<T>) -> Option<T> {
        let mut x = 0;
        let mut y = 0;
        let mut dx = 0;
        let mut dy = -1;
        // let o = (n as i32 % 2) ^ 1;
        let half = n as i32/2;
        for _ in 0..n.pow(2) {
            if x >= -half && x <= half && y >= -half && y <= half {
                // println!("{x}, {y}, {}, {}, {}, {}", x+half, y+half, x+half-o, y+half-o);
                let result = callback(x, y);
                if result.is_some() {return result};
            }
            if (x == y) || (x == -y && x < 0) || (x == 1-y && x > 0) {
                (dx, dy) = (-dy, dx);
            }
            x += dx;
            y += dy;
        }
        None
    }
}
use std::sync::{atomic::{AtomicU32, Ordering}, Arc};

use crate::bytes::AsFromBytes;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub id: u32,
}

impl Voxel {
    pub fn new(id: u32) -> Voxel {
        Voxel { id }
    }
}

impl AsFromBytes for Voxel {}


#[repr(C)]
#[derive(Debug)]
pub struct VoxelAtomic {
    pub id: AtomicU32,
}


impl VoxelAtomic {
    #[inline]
    pub fn to_voxel(&self) -> Voxel {
        Voxel::new(self.id())
    }

    #[inline]
    pub fn new(id: u32) -> Self {
        Self { id: AtomicU32::new(id) }
    }

    /// Update self in relaxed ordering
    #[inline]
    pub fn update(&self, id: u32) {
        self.id.store(id, Ordering::Relaxed);
    }

    /// Get id in relaxed ordering
    #[inline]
    pub fn id(&self) -> u32 {
        self.id.load(Ordering::Relaxed)
    }
}
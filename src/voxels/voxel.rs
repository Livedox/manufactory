#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub id: u32,
}

impl Voxel {
    pub fn new(id: u32) -> Voxel {
        Voxel { id }
    }
}
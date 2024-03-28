pub mod greed_manager;
pub(crate) mod face_managers;
pub mod default_manager;
use self::{greed_manager::GreedManager, default_manager::DefaultManager};
use super::{Buffer};
use crate::graphic::render::block::BlockFace;

pub enum BlockManagers {
    Greed(GreedManager),
    Default(DefaultManager),
}

impl BlockManagers {
    pub fn new(is_greed: bool) -> Self {
        match is_greed {
            true => Self::Greed(GreedManager::new()),
            false => Self::Default(DefaultManager::new()),
        }
    }

    pub fn set(&mut self, side: usize, layer: usize, row: usize, column: usize, face: BlockFace) {
        match self {
            Self::Greed(g) => g.set(side, layer, row, column, face),
            Self::Default(b) => b.set(side, layer, row, column, face),
        }
    }

    pub fn manage_vertices(&mut self, buffer: &mut Buffer, global: (f32, f32, f32)) {
        match self {
            Self::Greed(g) => g.manage_vertices(buffer, global),
            Self::Default(b) => b.manage_vertices(buffer, global),
        }
    }
}
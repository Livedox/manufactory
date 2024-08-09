use std::{error::Error, sync::Arc};

use egui::ahash::HashMap;

use crate::{coords::chunk_coord::ChunkCoord, voxels::chunk::Chunk};

use super::Server;

pub struct NoServer {
    chunks: HashMap<ChunkCoord, Option<Chunk>>,
}

impl NoServer {
    pub fn new() -> Self {
        todo!()
    }
}

impl Server for NoServer {
    fn load_chunk(cc: ChunkCoord) -> Result<Option<Arc<Chunk>>, Box<dyn Error>> {
        todo!()
    }
}
use std::{error::Error, sync::Arc};

use crate::{coords::chunk_coord::ChunkCoord, voxels::chunk::Chunk};
pub mod no_server;

pub trait Server {
    fn load_chunk(cc: ChunkCoord) -> Result<Option<Arc<Chunk>>, Box<dyn Error>>;
}
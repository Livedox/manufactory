use std::{error::Error, sync::Arc};
use std::fmt::Debug;

use crate::{coords::chunk_coord::ChunkCoord, voxels::chunk::Chunk};
pub mod no_server;
pub mod local_server;
pub mod connect_local_server;

pub trait Server {
    async fn test(&self) -> Result<Box<dyn Debug>, Box<dyn Error>>;
    async fn load_chunk(cc: ChunkCoord) -> Result<Option<Arc<Chunk>>, Box<dyn Error>>;
}
use std::collections::HashMap;
use std::{error::Error, sync::Arc};
use std::fmt::Debug;

use crate::{coords::chunk_coord::ChunkCoord, voxels::chunk::Chunk};

use super::Server;
pub struct ConnectLocalServer {

}

impl Server for ConnectLocalServer {
    fn load_chunk(cc: ChunkCoord) -> Result<Option<Arc<Chunk>>, Box<dyn Error>> {
        todo!()
    }

    fn test() -> Result<Box<dyn Debug>, Box<dyn Error>> {
        todo!()
    }
}
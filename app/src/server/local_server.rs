use tokio::net::UdpSocket;

use super::Server;
use std::collections::HashMap;
use std::{error::Error, sync::Arc};
use std::fmt::Debug;

use crate::{coords::chunk_coord::ChunkCoord, voxels::chunk::Chunk};
pub struct LocalServer {
    sock: Arc<UdpSocket>,
}

impl LocalServer {
    pub async fn new() -> Result<Self, std::io::Error>  {
        let sock = Arc::new(UdpSocket::bind("0.0.0.0:8080").await?);
        Self::run(Arc::clone(&sock)).await?;
        Ok(Self {
            sock,
        })
    }

    async fn run(socket: Arc<UdpSocket>) -> Result<(), std::io::Error> {
        let mut buf = [0; 1024];
        tokio::spawn(async move {loop {
            let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
            println!("{:?} bytes received from {:?}", len, addr);
        }});

        Ok(())
    }
}

impl Server for LocalServer {
    async fn load_chunk(cc: ChunkCoord) -> Result<Option<Arc<Chunk>>, Box<dyn Error>> {
        todo!()
    }

    async fn test(&self) -> Result<Box<dyn Debug>, Box<dyn Error>> {
        todo!()
    }
}
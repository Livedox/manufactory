use std::collections::HashMap;
use std::net::UdpSocket;
use std::{error::Error, sync::Arc};
use std::fmt::Debug;

use crate::{coords::chunk_coord::ChunkCoord, voxels::chunk::Chunk};

use super::Server;
pub struct ConnectLocalServer {
    sock: Arc<UdpSocket>,
}

impl ConnectLocalServer {
    pub async fn new() -> Result<Self, std::io::Error>  {
        let sock = Arc::new(UdpSocket::bind("0.0.0.0:8081")?);
        sock.connect("0.0.0.0:8080")?;
        Ok(Self {
            sock,
        })
    }

    // async fn run(socket: Arc<UdpSocket>) -> Result<(), std::io::Error> {
    //     let mut buf = [0; 1024];
    //     tokio::spawn(async move {loop {
    //         let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
    //         println!("{:?} bytes received from {:?}", len, addr);
    //     }});

    //     Ok(())
    // }
}

impl Server for ConnectLocalServer {
    async fn load_chunk(cc: ChunkCoord) -> Result<Option<Arc<Chunk>>, Box<dyn Error>> {
        todo!()
    }

    async fn test(&self) -> Result<Box<dyn Debug>, Box<dyn Error>> {
        self.sock.send(&[1, 2, 3, 4, 5])?;
        Ok(Box::new("Work"))
    }
}
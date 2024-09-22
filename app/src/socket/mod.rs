use client::{Client, ClientConfig};
use packet::{header::{Header, HeaderId, PROTOCOL_VERSION}, packet::SocketServerEvent};
use serde::{Deserialize, Serialize};
use server::{SocketServer, SocketServerConfig};
use tokio::{io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}, sync::mpsc::Receiver};

use std::{collections::HashMap, fmt::{write, Display}, io::{self, BufRead, Bytes}, net::SocketAddr, os::windows::io::{AsRawSocket, AsSocket}, sync::Arc, time::Duration};
pub mod packet;
pub mod server;
pub mod client;
pub mod common;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Test {
    A,
    V(Vec<u8>),
    G(String),
}

pub struct MessageHeader {
    pub id: u32,
    pub size: u32,
}

pub struct Message {
    pub header: MessageHeader,
    pub body: Vec<u8>,
}

impl Message {
    pub fn size(&self) -> usize {
        std::mem::size_of::<MessageHeader>() + self.body.len()
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message id: {} size: {}", self.header.id, self.header.size)
    }
}

async fn process_socket<T>(socket: T) {
    // do work with socket here
}

// pub async fn socket_test() -> io::Result<(SocketServer, Receiver<SocketServerEvent>, Receiver<Vec<u8>>)> {
//     let header = Header::new(HeaderId::Heartbeat, 2);
//     let num = <u64>::from(header);
//     let header_two = Header::try_from(num).unwrap();

//     assert_eq!(header, header_two);
    

//     let (tx1, rx2) = tokio::sync::mpsc::channel(200);
//     let mut _client = Client::start(ClientConfig::default(), tx1.clone()).await.unwrap();
//     // client.send_header(Header::new(HeaderId::Heartbeat, 0)).await.unwrap();

//     let mut _client2 = Client::start(ClientConfig::default(), tx1).await.unwrap();
//     for _ in 0..2 {
//         // client2.send_header(Header::new(HeaderId::Heartbeat, 0)).await.unwrap();
//     }

//     server.send_all(Arc::new([1u8,2,3,4,5]));
//     // drop(server);
//     // loop {}
//     Ok((server, rx, rx2))
// }
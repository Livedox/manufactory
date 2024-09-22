use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

use crate::socket::{common::{Handshake, HandshakeId}, packet::header::PROTOCOL_VERSION};

use super::packet::header::Header;

pub struct ClientConfig {
    addr: String,
    max_message_size: usize,
    max_packet_count: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            addr: String::from("127.0.0.1:8090"),
            max_message_size: 2_097_152, // 2 megabytes
            max_packet_count: 200,
        }
    }
}


pub struct Client {
    ptx: tokio::sync::mpsc::Sender<Vec<u8>>,
    id: u32,
}


impl Client {
    pub async fn start(config: ClientConfig, payload_sender: tokio::sync::mpsc::Sender<Vec<u8>>) -> tokio::io::Result<Self> {
        println!("a");
        let mut stream = TcpStream::connect(config.addr).await?;
        println!("b");
        let id = Self::handshake(&mut stream).await.unwrap();
        println!("c");
        let (ptx, mut prx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);

        let (mut reader, mut writer) = stream.into_split();

        tokio::spawn(async move {
            let mut buf: Vec<u8> = vec![0; 1_000_000];
            loop {
                let Ok(size) = reader.read_u32().await else {break};
                if size == 0 || size > 1_000_000 {break};
                let buf = &mut buf[0..size as usize];
                reader.read_exact(buf).await.unwrap();
                payload_sender.send(Vec::from(buf)).await.unwrap();
            }
        });

        tokio::spawn(async move {
            loop {
                let Some(data) = prx.recv().await else {break};
                let size = data.len() as u32;
                writer.write_u32(size).await.unwrap();
                writer.write_all(&data).await.unwrap();
            }
        });

        Ok(Self {ptx, id})
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn send_payload(&mut self, payload: Vec<u8>) {
        self.ptx.try_send(payload).unwrap();
    }


    async fn handshake(stream: &mut TcpStream) -> Result<u32, Box<dyn std::error::Error>> {
        stream.write_u64(Handshake::new(HandshakeId::ProtocolVersion, PROTOCOL_VERSION as u32).into()).await?;
        let handshake: Handshake = stream.read_u64().await?.into();
        Self::check_reject(stream, handshake).await?;
        if handshake.id() != HandshakeId::ClientId {
            return Err("Incorrect Handshake Protocol".into());
        }
        let client_id = handshake.data();
        let nickname = "Big_Cock!".as_bytes();
        println!("{:?}", nickname.len() as u32);
        stream.write_u64(Handshake::new(HandshakeId::Nickname, nickname.len() as u32).into()).await?;
        stream.write_all(nickname).await?;
        let handshake: Handshake = stream.read_u64().await?.into();
        Self::check_reject(stream, handshake).await?;
        if handshake.id() != HandshakeId::Ok {
            return Err("Incorrect Handshake Protocol".into());
        }
        println!("Cool handshake client!");
        Ok(client_id)
    }

    async fn check_reject(stream: &mut TcpStream, handshake: Handshake) -> Result<(), Box<dyn std::error::Error>> {
        if handshake.id() == HandshakeId::Reject {
            let mut reject_msg = String::new();
            stream.read_to_string(&mut reject_msg).await?;
            return Err(reject_msg.into())
        }
        Ok(())
    } 
}